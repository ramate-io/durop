//! AppKit focus policy without changing an initialized NSWindow's class.
//!
//! Winit's class-wide method adapters dispatch only marked overlay instances;
//! ordinary panel windows retain the original implementations and window ISA.
use super::{NativeReady, OverlayWindow};
use bevy::{ecs::system::NonSendMarker, prelude::*, window::RawHandleWrapper};
use objc2::{
    msg_send,
    runtime::{AnyClass, AnyObject, Bool, Imp, Method, Sel},
    sel,
};
use raw_window_handle::RawWindowHandle;
use std::{ffi::c_char, sync::OnceLock};

#[link(name = "objc")]
unsafe extern "C" {
    /// Declare the Objective-C runtime entry for a class-local selector override.
    fn class_replaceMethod(
        class: &AnyClass,
        selector: Sel,
        implementation: Imp,
        types: *const c_char,
    ) -> Option<Imp>;

    /// Read the original method ABI encoding before installing an adapter.
    fn method_getTypeEncoding(method: &Method) -> *const c_char;

    /// Associate a non-retaining marker with a live Objective-C window.
    fn objc_setAssociatedObject(
        object: &AnyObject,
        key: *const u8,
        value: &AnyObject,
        policy: usize,
    );

    /// Read the adapter-owned marker from a live Objective-C window.
    fn objc_getAssociatedObject(object: &AnyObject, key: *const u8) -> *mut AnyObject;
}

/// Unique stable address identifying windows owned by this overlay adapter.
static OVERLAY_KEY: u8 = 0;

/// Preserve native selector implementations for ordinary, unmarked windows.
struct Originals {
    /// Original canBecomeKeyWindow implementation.
    key: Imp,
    /// Original canBecomeMainWindow implementation.
    main: Imp,
    /// Original makeKeyAndOrderFront implementation.
    show: Imp,
}

/// Publish original implementations before installing replacement callbacks.
static ORIGINALS: OnceLock<Originals> = OnceLock::new();

/// Ensure class-wide selector adapters are installed once on the main thread.
static INSTALLED: OnceLock<()> = OnceLock::new();

/// Recognize only windows explicitly marked by this adapter.
fn is_overlay(window: &AnyObject) -> bool {
    // SAFETY: The stable key belongs exclusively to this adapter. An assign
    // association stores the live window itself, without retaining it in a cycle.
    unsafe { std::ptr::eq(objc_getAssociatedObject(window, &OVERLAY_KEY), window) }
}

/// Mark a live overlay without changing its class or creating a retain cycle.
unsafe fn mark_overlay(window: &AnyObject) {
    // SAFETY: Both object and value are this live window; policy 0 is ASSIGN.
    unsafe { objc_setAssociatedObject(window, &OVERLAY_KEY, window, 0) };
}

/// Deny key-window status only for marked overlays.
extern "C-unwind" fn can_become_key(window: &AnyObject, selector: Sel) -> Bool {
    focus_policy(window, selector, ORIGINALS.get().unwrap().key)
}

/// Deny main-window status only for marked overlays.
extern "C-unwind" fn can_become_main(window: &AnyObject, selector: Sel) -> Bool {
    focus_policy(window, selector, ORIGINALS.get().unwrap().main)
}

/// Apply the overlay focus exception or forward to the original selector.
fn focus_policy(window: &AnyObject, selector: Sel, original: Imp) -> Bool {
    if is_overlay(window) {
        Bool::NO
    } else {
        // SAFETY: Saved from the same BOOL-returning NSWindow selector; preserve
        // the receiver and selector, including ordinary panel behavior.
        let original: unsafe extern "C-unwind" fn(&AnyObject, Sel) -> Bool =
            unsafe { std::mem::transmute(original) };
        unsafe { original(window, selector) }
    }
}

/// Order overlays without focus while preserving normal panel window behavior.
extern "C-unwind" fn show(window: &AnyObject, selector: Sel, sender: *mut AnyObject) {
    if is_overlay(window) {
        // SAFETY: orderFront: preserves NSWindow ordering without taking focus.
        unsafe {
            let _: () = msg_send![window, orderFront: sender];
        }
    } else {
        // SAFETY: The saved implementation has the same selector ABI.
        let original: unsafe extern "C-unwind" fn(&AnyObject, Sel, *mut AnyObject) =
            unsafe { std::mem::transmute(ORIGINALS.get().unwrap().show) };
        unsafe { original(window, selector, sender) }
    }
}

/// Install ABI-compatible adapters on WinitWindow while preserving inherited NSWindow behavior.
unsafe fn install(class: &AnyClass) {
    INSTALLED.get_or_init(|| {
        let key = class.instance_method(sel!(canBecomeKeyWindow)).unwrap();
        let main = class.instance_method(sel!(canBecomeMainWindow)).unwrap();
        let order = class.instance_method(sel!(makeKeyAndOrderFront:)).unwrap();
        ORIGINALS.set(Originals {
            key: key.implementation(),
            main: main.implementation(),
            show: order.implementation(),
        }).unwrap_or_else(|_| panic!("window adapters already initialized"));
        // SAFETY: Each replacement has its selector's original ABI. Add an
        // override on WinitWindow itself even when the original is inherited:
        // changing NSWindow's inherited method would affect unrelated windows.
        // Publish originals before installing any callback, on the main thread.
        unsafe {
            for (selector, implementation, method) in [
                (sel!(canBecomeKeyWindow), std::mem::transmute::<extern "C-unwind" fn(&AnyObject, Sel) -> Bool, Imp>(can_become_key), key),
                (sel!(canBecomeMainWindow), std::mem::transmute::<extern "C-unwind" fn(&AnyObject, Sel) -> Bool, Imp>(can_become_main), main),
                (sel!(makeKeyAndOrderFront:), std::mem::transmute::<extern "C-unwind" fn(&AnyObject, Sel, *mut AnyObject), Imp>(show), order),
            ] {
                class_replaceMethod(class, selector, implementation, method_getTypeEncoding(method));
            }
        }
    });
}

/// Apply click-through, nonactivating AppKit policy to newly created overlay windows.
#[expect(
    clippy::type_complexity,
    reason = "Bevy query filters keep the native window boundary explicit"
)]
pub(super) fn configure(
    mut commands: Commands,
    windows: Query<(Entity, &RawHandleWrapper), (With<OverlayWindow>, Without<NativeReady>)>,
    _main_thread: NonSendMarker,
) {
    for (entity, handle) in &windows {
        let RawWindowHandle::AppKit(handle) = handle.get_window_handle() else {
            continue;
        };
        // SAFETY: Bevy owns the live view, and NonSendMarker keeps this on the
        // AppKit thread. Preserve the existing ISA and AppKit computed-property
        // metadata; replacing an initialized window's class crashes on macOS 27.
        unsafe {
            let view = handle.ns_view.as_ptr().cast::<AnyObject>();
            let window: *mut AnyObject = msg_send![view, window];
            let Some(window) = window.as_ref() else {
                continue;
            };
            let class = AnyClass::get(c"WinitWindow").expect("winit 0.30 window class");
            let compatible: Bool = msg_send![window, isKindOfClass: class];
            assert!(
                compatible.as_bool(),
                "review native adapter after winit class changes"
            );
            install(class);
            mark_overlay(window);
            let _: () = msg_send![window, setIgnoresMouseEvents: Bool::YES];
            let _: () = msg_send![window, setHasShadow: Bool::NO];
            let _: () = msg_send![window, setHidesOnDeactivate: Bool::NO];
            // CanJoinAllSpaces | Stationary | IgnoresCycle | FullScreenAuxiliary.
            let _: () = msg_send![window, setCollectionBehavior: (1_usize | 16 | 64 | 256)];
        }
        commands.entity(entity).insert(NativeReady);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use objc2::{rc::Retained, runtime::ClassBuilder};
    use std::sync::atomic::{AtomicUsize, Ordering};

    static ORIGINAL_SHOWS: AtomicUsize = AtomicUsize::new(0);
    static NONACTIVATING_SHOWS: AtomicUsize = AtomicUsize::new(0);

    extern "C-unwind" fn original_focus(_: &AnyObject, _: Sel) -> Bool {
        Bool::YES
    }
    extern "C-unwind" fn original_show(_: &AnyObject, _: Sel, _: *mut AnyObject) {
        ORIGINAL_SHOWS.fetch_add(1, Ordering::Relaxed);
    }
    extern "C-unwind" fn order_front(_: &AnyObject, _: Sel, _: *mut AnyObject) {
        NONACTIVATING_SHOWS.fetch_add(1, Ordering::Relaxed);
    }

    #[test]
    fn marked_overlay_preserves_isa_and_ordinary_window_dispatch() {
        // NSObject probes exercise Objective-C method lookup and association
        // dispatch without requiring an AppKit event loop on the test thread.
        // Put originals on the parent to catch accidentally replacing inherited
        // NSWindow methods instead of adding overrides on WinitWindow itself.
        unsafe {
            let mut parent = ClassBuilder::new(
                c"DuropFocusPolicyProbeParent",
                AnyClass::get(c"NSObject").unwrap(),
            )
            .unwrap();
            parent.add_method(
                sel!(canBecomeKeyWindow),
                original_focus as extern "C-unwind" fn(_, _) -> _,
            );
            parent.add_method(
                sel!(canBecomeMainWindow),
                original_focus as extern "C-unwind" fn(_, _) -> _,
            );
            parent.add_method(
                sel!(makeKeyAndOrderFront:),
                original_show as extern "C-unwind" fn(_, _, _),
            );
            parent.add_method(
                sel!(orderFront:),
                order_front as extern "C-unwind" fn(_, _, _),
            );
            let parent = parent.register();
            let class = ClassBuilder::new(c"DuropFocusPolicyProbe", parent)
                .unwrap()
                .register();
            let ordinary: Retained<AnyObject> = msg_send![class, new];
            let overlay: Retained<AnyObject> = msg_send![class, new];
            let unrelated: Retained<AnyObject> = msg_send![parent, new];
            install(class);
            mark_overlay(&overlay);
            assert!(std::ptr::eq(overlay.class(), class));
            assert!(std::ptr::eq(ordinary.class(), class));
            for object in [&*ordinary, &*unrelated] {
                let key: Bool = msg_send![object, canBecomeKeyWindow];
                let main: Bool = msg_send![object, canBecomeMainWindow];
                assert!(key.as_bool() && main.as_bool());
                let _: () =
                    msg_send![object, makeKeyAndOrderFront: std::ptr::null_mut::<AnyObject>()];
            }
            let key: Bool = msg_send![&*overlay, canBecomeKeyWindow];
            let main: Bool = msg_send![&*overlay, canBecomeMainWindow];
            assert!(!key.as_bool() && !main.as_bool());
            let _: () =
                msg_send![&*overlay, makeKeyAndOrderFront: std::ptr::null_mut::<AnyObject>()];
            assert_eq!(ORIGINAL_SHOWS.load(Ordering::Relaxed), 2);
            assert_eq!(NONACTIVATING_SHOWS.load(Ordering::Relaxed), 1);
        }
    }
}
