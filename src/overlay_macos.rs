//! A fresh nonactivating NSPanel owns the overlay surface; winit keeps its hidden event window.
//!
//! Never reclass or reparent winit's initialized NSWindow/NSView. Its callbacks assume
//! their original concrete classes. Only the renderer's raw view handle is redirected.
use super::{NativeReady, OverlayCamera, OverlayState, OverlayWindow};
use bevy::{
    prelude::*,
    window::{RawHandleWrapper, RawHandleWrapperHolder},
    winit::WinitMonitors,
};
use objc2::{
    msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel},
    sel,
};
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
use raw_window_handle::{AppKitWindowHandle, RawWindowHandle};
use std::{ptr::NonNull, sync::OnceLock};
use winit::platform::macos::MonitorHandleExtMacOS;

#[link(name = "objc")]
unsafe extern "C" {
    /// Attach panel ownership to the original window retained by Bevy's render handle.
    fn objc_setAssociatedObject(
        object: &AnyObject,
        key: *const u8,
        value: &AnyObject,
        policy: usize,
    );
}

#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    /// Resolve public system levels rather than relying on their current numeric values.
    fn CGWindowLevelForKey(key: i32) -> i32;
}

/// Retaining association keeps the panel/view alive through pipelined GPU surface teardown.
static PANEL_KEY: u8 = 0;

/// Register the subclass before allocation; initialized AppKit objects never change class.
static PANEL_CLASS: OnceLock<&'static AnyClass> = OnceLock::new();

/// AppKit must never give overlay content keyboard or application-main status.
extern "C-unwind" fn refuses_focus(_: &AnyObject, _: Sel) -> Bool {
    Bool::NO
}

/// Full display coverage deliberately includes the Dock/menu area, unlike the visible work area.
extern "C-unwind" fn unconstrained_frame(
    _: &AnyObject,
    _: Sel,
    frame: NSRect,
    _: *mut AnyObject,
) -> NSRect {
    frame
}

/// Main-thread ownership and presentation state for the single selected display.
#[derive(Default)]
pub(super) struct NativeOverlay {
    /// An additional retain for main-thread operations; the raw handle owner also retains this panel.
    panel: Option<Retained<AnyObject>>,
    /// Last applied full-screen frame avoids redundant AppKit resizing each render tick.
    frame: Option<NSRect>,
    /// Whether the native panel is currently ordered in.
    visible: bool,
}

impl NativeOverlay {
    /// Construct a genuine nonactivating panel and plain render view on the AppKit thread.
    unsafe fn create(owner: &AnyObject) -> Result<(Self, NonNull<std::ffi::c_void>), &'static str> {
        let parent = AnyClass::get(c"NSPanel").ok_or("NSPanel unavailable")?;
        let class = PANEL_CLASS.get_or_init(|| {
            let mut builder =
                ClassBuilder::new(c"DuropOverlayPanel", parent).expect("unique overlay class");
            // SAFETY: The methods exactly match their inherited NSWindow selector ABIs.
            unsafe {
                builder.add_method(
                    sel!(canBecomeKeyWindow),
                    refuses_focus as extern "C-unwind" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(canBecomeMainWindow),
                    refuses_focus as extern "C-unwind" fn(_, _) -> _,
                );
                builder.add_method(
                    sel!(constrainFrameRect:toScreen:),
                    unconstrained_frame as extern "C-unwind" fn(_, _, _, _) -> _,
                );
            }
            builder.register()
        });
        let rect = NSRect::new(NSPoint::new(0., 0.), NSSize::new(1., 1.));
        // SAFETY: Allocate the final class with NonactivatingPanel (1<<7) from the outset.
        // The borderless panel owns a plain NSView, never winit's NSView subclass.
        unsafe {
            let allocated: Allocated<AnyObject> = msg_send![*class, alloc];
            let panel: Retained<AnyObject> = msg_send![allocated, initWithContentRect: rect, styleMask: 1_usize << 7, backing: 2_usize, defer: Bool::NO];
            let view_class = AnyClass::get(c"NSView").ok_or("NSView unavailable")?;
            let allocated: Allocated<AnyObject> = msg_send![view_class, alloc];
            let view: Retained<AnyObject> = msg_send![allocated, initWithFrame: rect];
            let _: () = msg_send![&*view, setWantsLayer: Bool::YES];
            let _: () = msg_send![&*panel, setContentView: &*view];
            let _: () = msg_send![&*panel, setReleasedWhenClosed: Bool::NO];
            let _: () = msg_send![&*panel, setOpaque: Bool::NO];
            let color_class = AnyClass::get(c"NSColor").ok_or("NSColor unavailable")?;
            let clear: *mut AnyObject = msg_send![color_class, clearColor];
            let _: () = msg_send![&*panel, setBackgroundColor: clear];
            let _: () = msg_send![&*panel, setIgnoresMouseEvents: Bool::YES];
            let _: () = msg_send![&*panel, setHasShadow: Bool::NO];
            let _: () = msg_send![&*panel, setHidesOnDeactivate: Bool::NO];
            let _: () = msg_send![&*panel, setFloatingPanel: Bool::YES];
            let _: () = msg_send![&*panel, setBecomesKeyOnlyIfNeeded: Bool::YES];
            let _: () = msg_send![&*panel, setAnimationBehavior: 2_isize]; // NSWindowAnimationBehaviorNone
            // CanJoinAllSpaces | Stationary | IgnoresCycle | FullScreenAuxiliary.
            let _: () = msg_send![&*panel, setCollectionBehavior: (1_usize | 16 | 64 | 256)];
            let level = CGWindowLevelForKey(9); // kCGStatusWindowLevelKey, above the Dock.
            let _: () = msg_send![&*panel, setLevel: level as isize];
            let title = NSString::from_str("Durop native overlay");
            let _: () = msg_send![&*panel, setTitle: &*title];
            // RETAIN_NONATOMIC: winit releases its native owner on the main thread;
            // RawHandleWrapper retains winit through all extracted render references.
            objc_setAssociatedObject(owner, &PANEL_KEY, &panel, 1);
            let pointer = NonNull::from(&*view).cast();
            eprintln!("Native overlay: nonactivating NSPanel, window level {level}");
            Ok((
                Self {
                    panel: Some(panel),
                    ..default()
                },
                pointer,
            ))
        }
    }

    /// Hide without closing/releasing a view that an in-flight surface may still reference.
    fn hide(&mut self) {
        if self.visible {
            if let Some(panel) = &self.panel {
                // SAFETY: NativeOverlay is a NonSend resource, accessed/dropped on the main thread.
                unsafe {
                    let _: () = msg_send![&**panel, orderOut: std::ptr::null_mut::<AnyObject>()];
                }
            }
            self.visible = false;
        }
    }
}

impl Drop for NativeOverlay {
    fn drop(&mut self) {
        self.hide();
    }
}

/// Redirect the render handle before the first surface extraction; keep winit's owner hidden.
#[expect(
    clippy::type_complexity,
    reason = "Native boundary selects only unconfigured overlay handles"
)]
pub(super) fn configure(
    mut commands: Commands,
    mut windows: Query<
        (
            Entity,
            &mut RawHandleWrapper,
            Option<&RawHandleWrapperHolder>,
        ),
        (With<OverlayWindow>, Without<NativeReady>),
    >,
    mut native: NonSendMut<NativeOverlay>,
) {
    for (entity, mut handle, holder) in &mut windows {
        let RawWindowHandle::AppKit(original) = handle.get_window_handle() else {
            continue;
        };
        // SAFETY: This runs on the AppKit thread. The original raw handle retains its
        // winit owner, which retains the new panel via association for the surface lifetime.
        unsafe {
            let view = original.ns_view.as_ptr().cast::<AnyObject>();
            let owner: *mut AnyObject = msg_send![view, window];
            let Some(owner) = owner.as_ref() else {
                continue;
            };
            match NativeOverlay::create(owner) {
                Ok((panel, view)) => {
                    handle
                        .set_window_handle(RawWindowHandle::AppKit(AppKitWindowHandle::new(view)));
                    if let Some(holder) = holder {
                        *holder.0.lock().unwrap() = Some(handle.clone());
                    }
                    *native = panel;
                    commands.entity(entity).insert(NativeReady);
                }
                Err(message) => eprintln!("Native overlay unavailable: {message}"),
            }
        }
    }
}

/// Follow the selected NSScreen's full bounds without moving focus or activating the application.
pub(super) fn synchronize(
    state: Res<OverlayState>,
    monitors: Res<WinitMonitors>,
    camera: Single<&Camera, With<OverlayCamera>>,
    mut native: NonSendMut<NativeOverlay>,
) {
    let display = state
        .target
        .and_then(|entity| monitors.find_entity(entity))
        .map(|monitor| monitor.native_id());
    let frame = display.and_then(|display| {
        // SAFETY: NSScreen/NSArray/NSDictionary access is on the AppKit thread;
        // deviceDescription's NSScreenNumber is the public CGDirectDisplayID bridge.
        unsafe {
            let screens: *mut AnyObject = msg_send![AnyClass::get(c"NSScreen")?, screens];
            let count: usize = msg_send![screens, count];
            let key = NSString::from_str("NSScreenNumber");
            for index in 0..count {
                let screen: *mut AnyObject = msg_send![screens, objectAtIndex: index];
                let description: *mut AnyObject = msg_send![screen, deviceDescription];
                let number: *mut AnyObject = msg_send![description, objectForKey: &*key];
                let id: u32 = msg_send![number, unsignedIntValue];
                if id == display {
                    return Some(msg_send![screen, frame]);
                }
            }
            None
        }
    });
    if !camera.is_active || frame.is_none() {
        native.hide();
        return;
    }
    let Some(panel) = native.panel.as_ref() else {
        return;
    };
    let frame = frame.unwrap();
    // SAFETY: Never use makeKey*, activate*, or application-wide presentation options.
    unsafe {
        if native.frame != Some(frame) {
            let _: () = msg_send![&**panel, setFrame: frame, display: Bool::YES];
        }
        if !native.visible {
            let _: () = msg_send![&**panel, orderFrontRegardless];
        }
    }
    native.frame = Some(frame);
    native.visible = true;
}
