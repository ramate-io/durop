//! Dock reopen delivers an application action; no native window pointer escapes.
use bevy::{
    ecs::system::NonSendMarker,
    prelude::*,
    winit::{EventLoopProxyWrapper, WinitUserEvent},
};
use objc2::{
    msg_send,
    runtime::{AnyClass, AnyObject, Bool, Imp, Sel},
    sel,
};
use std::sync::{
    OnceLock,
    atomic::{AtomicBool, Ordering},
};

/// Latch Dock reopen callbacks until the Bevy application update handles them.
static REOPEN: AtomicBool = AtomicBool::new(false);

/// Wake the event loop when an AppKit callback arrives while idle.
static WAKE: OnceLock<Box<dyn Fn() + Send + Sync>> = OnceLock::new();
#[link(name = "objc")]
unsafe extern "C" {
    /// Declare the Objective-C runtime entry used to add the optional reopen selector.
    fn class_addMethod(
        class: &AnyClass,
        selector: Sel,
        implementation: Imp,
        types: *const std::ffi::c_char,
    ) -> Bool;
}

/// Queue a Dock reopen request without retaining native objects beyond the callback.
extern "C-unwind" fn reopen(_: &AnyObject, _: Sel, _: &AnyObject, _: Bool) -> Bool {
    REOPEN.store(true, Ordering::Release);
    if let Some(wake) = WAKE.get() {
        wake();
    }
    Bool::YES
}

/// Register the optional application delegate selector on the AppKit thread.
pub fn install(
    proxy: Res<EventLoopProxyWrapper>,
    mut support: ResMut<crate::app::ReopenSupport>,
    _main: NonSendMarker,
) {
    let proxy = (**proxy).clone();
    let _ = WAKE.set(Box::new(move || {
        let _ = proxy.send_event(WinitUserEvent::WakeUp);
    }));
    // SAFETY: AppKit and its existing delegate are accessed on the main thread.
    // Add only the optional reopen selector, preserving all existing delegate
    // methods. Refuse to override an implementation owned by another component.
    unsafe {
        let class = AnyClass::get(c"NSApplication").expect("AppKit application class");
        let application: *mut AnyObject = msg_send![class, sharedApplication];
        let delegate: *mut AnyObject = msg_send![application, delegate];
        if let Some(delegate) = delegate.as_ref() {
            let selector = sel!(applicationShouldHandleReopen:hasVisibleWindows:);
            let class = delegate.class();
            if class.instance_method(selector).is_none() {
                #[cfg(target_arch = "aarch64")]
                let encoding = c"B@:@B";
                #[cfg(not(target_arch = "aarch64"))]
                let encoding = c"c@:@c";
                let implementation = std::mem::transmute::<
                    extern "C-unwind" fn(&AnyObject, Sel, &AnyObject, Bool) -> Bool,
                    Imp,
                >(reopen);
                support.0 =
                    class_addMethod(class, selector, implementation, encoding.as_ptr()).as_bool();
            }
        }
    }
}

/// Transfer a latched Dock request into the normal application action queue.
pub fn poll(mut actions: ResMut<crate::app::Actions>) {
    if REOPEN.swap(false, Ordering::AcqRel) {
        actions.0.push_back(crate::app::Action::Show);
    }
}

/// Write private connection details only after the host explicitly chooses Copy.
pub fn copy_text(text: &str) -> bool {
    unsafe {
        let Some(class) = AnyClass::get(c"NSPasteboard") else {
            return false;
        };
        let board: *mut AnyObject = msg_send![class, generalPasteboard];
        if board.is_null() {
            return false;
        }
        let _: isize = msg_send![board, clearContents];
        let kind = objc2_foundation::NSString::from_str("public.utf8-plain-text");
        let text = objc2_foundation::NSString::from_str(text);
        let copied: objc2::runtime::Bool = msg_send![board, setString: &*text, forType: &*kind];
        copied.as_bool()
    }
}
