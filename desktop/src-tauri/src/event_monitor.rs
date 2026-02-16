//! Global input event monitoring via macOS CGEventTap.
//!
//! Spawns a background thread that listens for mouse and keyboard events
//! and pushes them into the shared `EventBuffer`. Requires Accessibility
//! permissions to receive events.

use crate::events::{EventBuffer, InputEvent};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use std::ffi::c_void;

    extern "C" {
        fn CGEventTapCreate(
            tap: u32,        // CGEventTapLocation
            place: u32,      // CGEventTapPlacement
            options: u32,    // CGEventTapOptions
            events_of_interest: u64,
            callback: extern "C" fn(
                proxy: *const c_void,
                event_type: u32,
                event: *const c_void,
                user_info: *mut c_void,
            ) -> *const c_void,
            user_info: *mut c_void,
        ) -> *const c_void;

        fn CFMachPortCreateRunLoopSource(
            allocator: *const c_void,
            port: *const c_void,
            order: i64,
        ) -> *const c_void;

        fn CFRunLoopGetCurrent() -> *const c_void;
        fn CFRunLoopAddSource(rl: *const c_void, source: *const c_void, mode: *const c_void);
        fn CFRunLoopRun();
        fn CFRelease(cf: *const c_void);

        static kCFRunLoopCommonModes: *const c_void;

        fn CGEventGetLocation(event: *const c_void) -> CGPoint;
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    fn now_ts() -> f64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64()
    }

    // Event type constants
    const K_CG_EVENT_LEFT_MOUSE_DOWN: u32 = 1;
    const K_CG_EVENT_RIGHT_MOUSE_DOWN: u32 = 3;
    const K_CG_EVENT_KEY_DOWN: u32 = 10;
    const K_CG_EVENT_SCROLL_WHEEL: u32 = 22;
    const K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT: u32 = 0xFFFFFFFE;

    // Bitmask for events we want to listen to
    fn event_mask() -> u64 {
        (1 << K_CG_EVENT_LEFT_MOUSE_DOWN)
            | (1 << K_CG_EVENT_RIGHT_MOUSE_DOWN)
            | (1 << K_CG_EVENT_KEY_DOWN)
            | (1 << K_CG_EVENT_SCROLL_WHEEL)
    }

    struct TapContext {
        buffer: Arc<EventBuffer>,
        rt: tokio::runtime::Handle,
    }

    extern "C" fn tap_callback(
        _proxy: *const c_void,
        event_type: u32,
        event: *const c_void,
        user_info: *mut c_void,
    ) -> *const c_void {
        if event_type == K_CG_EVENT_TAP_DISABLED_BY_TIMEOUT {
            return event;
        }

        let ctx = unsafe { &*(user_info as *const TapContext) };
        let loc = unsafe { CGEventGetLocation(event) };

        let input_event = match event_type {
            K_CG_EVENT_LEFT_MOUSE_DOWN => InputEvent {
                event_type: "click".into(),
                x: Some(loc.x),
                y: Some(loc.y),
                key: None,
                modifiers: vec![],
                timestamp: now_ts(),
            },
            K_CG_EVENT_RIGHT_MOUSE_DOWN => InputEvent {
                event_type: "click".into(),
                x: Some(loc.x),
                y: Some(loc.y),
                key: Some("right".into()),
                modifiers: vec![],
                timestamp: now_ts(),
            },
            K_CG_EVENT_KEY_DOWN => {
                // Get the keycode from the CGEvent
                let keycode = unsafe {
                    extern "C" {
                        fn CGEventGetIntegerValueField(
                            event: *const c_void,
                            field: u32,
                        ) -> i64;
                    }
                    CGEventGetIntegerValueField(event, 9) // kCGKeyboardEventKeycode = 9
                };
                InputEvent {
                    event_type: "keypress".into(),
                    x: None,
                    y: None,
                    key: Some(keycode_to_name(keycode as u16)),
                    modifiers: vec![],
                    timestamp: now_ts(),
                }
            }
            K_CG_EVENT_SCROLL_WHEEL => InputEvent {
                event_type: "scroll".into(),
                x: Some(loc.x),
                y: Some(loc.y),
                key: None,
                modifiers: vec![],
                timestamp: now_ts(),
            },
            _ => return event,
        };

        let buf = ctx.buffer.clone();
        ctx.rt.spawn(async move {
            buf.push(input_event).await;
        });

        event
    }

    fn keycode_to_name(code: u16) -> String {
        match code {
            0x00 => "a", 0x01 => "s", 0x02 => "d", 0x03 => "f",
            0x04 => "h", 0x05 => "g", 0x06 => "z", 0x07 => "x",
            0x08 => "c", 0x09 => "v", 0x0B => "b", 0x0C => "q",
            0x0D => "w", 0x0E => "e", 0x0F => "r", 0x10 => "y",
            0x11 => "t", 0x1F => "o", 0x20 => "u", 0x22 => "i",
            0x23 => "p", 0x25 => "l", 0x26 => "j", 0x28 => "k",
            0x2D => "n", 0x2E => "m",
            0x24 => "return", 0x30 => "tab", 0x31 => "space",
            0x33 => "delete", 0x35 => "escape",
            0x7B => "left", 0x7C => "right", 0x7D => "down", 0x7E => "up",
            _ => "unknown",
        }
        .to_string()
    }

    pub fn start_event_monitor(buffer: Arc<EventBuffer>) {
        let rt_handle = tokio::runtime::Handle::current();

        std::thread::spawn(move || {
            let ctx = Box::new(TapContext {
                buffer,
                rt: rt_handle,
            });
            let ctx_ptr = Box::into_raw(ctx) as *mut c_void;

            let tap = unsafe {
                CGEventTapCreate(
                    0, // kCGHIDEventTap
                    0, // kCGHeadInsertEventTap
                    1, // kCGEventTapOptionListenOnly
                    event_mask(),
                    tap_callback,
                    ctx_ptr,
                )
            };

            if tap.is_null() {
                log::error!(
                    "Failed to create CGEventTap — \
                     grant Accessibility permission in System Settings"
                );
                return;
            }

            unsafe {
                let source = CFMachPortCreateRunLoopSource(std::ptr::null(), tap, 0);
                let run_loop = CFRunLoopGetCurrent();
                CFRunLoopAddSource(run_loop, source, kCFRunLoopCommonModes);
                CFRelease(source);

                log::info!("Event monitor started (CGEventTap active)");
                CFRunLoopRun();
            }
        });
    }
}

#[cfg(target_os = "macos")]
pub use macos::start_event_monitor;

#[cfg(not(target_os = "macos"))]
pub fn start_event_monitor(_buffer: Arc<EventBuffer>) {
    log::warn!("Event monitoring not supported on this platform");
}
