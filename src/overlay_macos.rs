//! The only AppKit boundary: a per-overlay subclass of winit's existing class.
//!
//! Do not swizzle winit globally: the normal panel must retain normal key-window
//! behavior. The subclass adds no ivars and preserves all inherited winit methods.
use super::{NativeReady, OverlayWindow};
use bevy::{ecs::system::NonSendMarker, prelude::*, window::RawHandleWrapper};
use objc2::{
    msg_send,
    runtime::{AnyClass, AnyObject, Bool, ClassBuilder, Sel},
    sel,
};
use raw_window_handle::RawWindowHandle;
use std::sync::OnceLock;

#[link(name = "objc")]
unsafe extern "C" {
    fn object_setClass(object: *mut AnyObject, class: *const AnyClass) -> *const AnyClass;
}

extern "C" fn cannot_focus(_: &AnyObject, _: Sel) -> Bool {
    Bool::NO
}
extern "C" fn show_without_activation(window: &AnyObject, _: Sel, sender: *mut AnyObject) {
    // SAFETY: installed only on NSWindow subclasses; preserves selector signature.
    unsafe {
        let _: () = msg_send![window, orderFront: sender];
    }
}

#[expect(
    clippy::type_complexity,
    reason = "Bevy query filters keep the native window boundary explicit"
)]
pub(super) fn configure(
    mut commands: Commands,
    windows: Query<(Entity, &RawHandleWrapper), (With<OverlayWindow>, Without<NativeReady>)>,
    // Force the system onto the AppKit main thread.
    _main_thread: NonSendMarker,
) {
    static CLASS: OnceLock<&'static AnyClass> = OnceLock::new();
    for (entity, handle) in &windows {
        let RawWindowHandle::AppKit(handle) = handle.get_window_handle() else {
            continue;
        };
        // SAFETY: Bevy owns the live raw view for the queried Window, and the
        // non-send parameter guarantees execution on the winit/AppKit thread.
        unsafe {
            let view = handle.ns_view.as_ptr().cast::<AnyObject>();
            let window: *mut AnyObject = msg_send![view, window];
            let Some(window_ref) = window.as_ref() else {
                continue;
            };
            let class = CLASS.get_or_init(|| {
                let mut class = ClassBuilder::new(c"DuropNonactivatingOverlay", window_ref.class())
                    .expect("unique overlay Objective-C class");
                class.add_method(
                    sel!(canBecomeKeyWindow),
                    cannot_focus as extern "C" fn(_, _) -> _,
                );
                class.add_method(
                    sel!(canBecomeMainWindow),
                    cannot_focus as extern "C" fn(_, _) -> _,
                );
                // winit 0.30 uses this selector for set_visible(true). Keep its
                // ordinary panel implementation intact, but never activate here.
                class.add_method(
                    sel!(makeKeyAndOrderFront:),
                    show_without_activation as extern "C" fn(_, _, _),
                );
                class.register()
            });
            object_setClass(window, *class);
            let _: () = msg_send![window, setIgnoresMouseEvents: Bool::YES];
            let _: () = msg_send![window, setHasShadow: Bool::NO];
            let _: () = msg_send![window, setHidesOnDeactivate: Bool::NO];
            // NSWindowCollectionBehavior: CanJoinAllSpaces | Stationary |
            // IgnoresCycle | FullScreenAuxiliary. Native fullscreen/game behavior
            // remains acceptance-test evidence, not a guarantee from these flags.
            let _: () = msg_send![window, setCollectionBehavior: (1_usize | 16 | 64 | 256)];
        }
        commands.entity(entity).insert(NativeReady);
    }
}
