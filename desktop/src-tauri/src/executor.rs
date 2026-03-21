//! Action executor: posts synthetic mouse/keyboard events via macOS CGEvent API.

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Action {
    #[serde(rename = "type")]
    pub action_type: String,
    pub coordinates: Option<Vec<f64>>,
    pub text: Option<String>,
    pub key: Option<String>,
    pub modifiers: Option<Vec<String>>,
    /// Non-None when the server denies the request (quota exceeded, payment required, etc.)
    pub error: Option<String>,
}

#[cfg(target_os = "macos")]
mod macos {
    use core_graphics::event::{
        CGEvent, CGEventTapLocation, CGEventType, CGKeyCode, CGMouseButton,
        CGEventFlags,
    };
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};
    use core_graphics::geometry::CGPoint;

    fn make_source() -> CGEventSource {
        CGEventSource::new(CGEventSourceStateID::HIDSystemState)
            .expect("Failed to create CGEventSource")
    }

    /// Move mouse and click at (x, y).
    pub fn click(x: f64, y: f64) -> Result<(), String> {
        let point = CGPoint::new(x, y);
        let source = make_source();

        let move_event = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::MouseMoved,
            point,
            CGMouseButton::Left,
        )
        .map_err(|_| "Failed to create mouse move event".to_string())?;
        move_event.post(CGEventTapLocation::HID);

        let down = CGEvent::new_mouse_event(
            source.clone(),
            CGEventType::LeftMouseDown,
            point,
            CGMouseButton::Left,
        )
        .map_err(|_| "Failed to create mouse down event".to_string())?;
        down.post(CGEventTapLocation::HID);

        let up = CGEvent::new_mouse_event(
            source,
            CGEventType::LeftMouseUp,
            point,
            CGMouseButton::Left,
        )
        .map_err(|_| "Failed to create mouse up event".to_string())?;
        up.post(CGEventTapLocation::HID);

        Ok(())
    }

    /// Type a string by posting keyboard events for each character.
    ///
    /// We use the keycode 0 (key 'a') as a carrier and override the
    /// Unicode string on the event so the correct character is typed
    /// regardless of keyboard layout.
    ///
    /// Explicitly clears all modifier flags so residual Command/Shift/etc.
    /// from prior hotkey actions don't turn letters into shortcuts.
    pub fn type_text(text: &str) -> Result<(), String> {
        use foreign_types_shared::ForeignType;

        let source = make_source();

        for ch in text.chars() {
            // Key down with unicode character — clear all modifier flags
            let down = CGEvent::new_keyboard_event(source.clone(), 0, true)
                .map_err(|_| "Failed to create key-down event".to_string())?;
            down.set_flags(CGEventFlags::CGEventFlagNull);

            // Set the Unicode string on the event via CoreGraphics C API
            let chars = [ch as u16];
            unsafe {
                CGEventKeyboardSetUnicodeString(
                    down.as_ptr(),
                    chars.len() as _,
                    chars.as_ptr(),
                );
            }
            down.post(CGEventTapLocation::HID);

            // Key up — also clear flags
            let up = CGEvent::new_keyboard_event(source.clone(), 0, false)
                .map_err(|_| "Failed to create key-up event".to_string())?;
            up.set_flags(CGEventFlags::CGEventFlagNull);
            up.post(CGEventTapLocation::HID);

            // Small delay between characters to avoid overwhelming the event system
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        Ok(())
    }

    // CGEventKeyboardSetUnicodeString is not exposed by the core-graphics crate,
    // so we link it directly from the CoreGraphics framework.
    extern "C" {
        fn CGEventKeyboardSetUnicodeString(
            event: *mut core_graphics::sys::CGEvent,
            string_length: u64,
            unicode_string: *const u16,
        );
    }

    /// Press a single key (by name) with optional modifiers.
    ///
    /// Properly releases modifier flags on key-up and sends explicit
    /// modifier key-up events so the system doesn't think they're stuck.
    pub fn keypress(key: &str, modifiers: &[String]) -> Result<(), String> {
        let source = make_source();
        let keycode = key_name_to_code(key);
        let flags = modifiers_to_flags(modifiers);

        // Key down with modifiers held
        let down = CGEvent::new_keyboard_event(source.clone(), keycode, true)
            .map_err(|_| "Failed to create keypress-down event".to_string())?;
        down.set_flags(flags);
        down.post(CGEventTapLocation::HID);

        // Key up with flags cleared
        let up = CGEvent::new_keyboard_event(source.clone(), keycode, false)
            .map_err(|_| "Failed to create keypress-up event".to_string())?;
        up.set_flags(CGEventFlags::CGEventFlagNull);
        up.post(CGEventTapLocation::HID);

        // Send explicit modifier key-up events to ensure they're released
        for m in modifiers {
            let mod_keycode: CGKeyCode = match m.to_lowercase().as_str() {
                "cmd" | "command" => 0x37,   // kVK_Command
                "shift" => 0x38,              // kVK_Shift
                "alt" | "option" => 0x3A,     // kVK_Option
                "ctrl" | "control" => 0x3B,   // kVK_Control
                _ => continue,
            };
            let mod_up = CGEvent::new_keyboard_event(source.clone(), mod_keycode, false)
                .map_err(|_| "Failed to create modifier-up event".to_string())?;
            mod_up.set_flags(CGEventFlags::CGEventFlagNull);
            mod_up.post(CGEventTapLocation::HID);
        }

        Ok(())
    }

    /// Scroll by a delta. Positive = scroll down.
    pub fn scroll(delta_y: i32) -> Result<(), String> {
        use foreign_types_shared::ForeignType;

        // CGEventCreateScrollWheelEvent is not directly in the crate,
        // so we use the C function.
        let source = make_source();
        let event = unsafe {
            let cg_event = CGEventCreateScrollWheelEvent(
                source.as_ptr(),
                0, // kCGScrollEventUnitPixel
                1, // wheel_count
                delta_y,
            );
            if cg_event.is_null() {
                return Err("Failed to create scroll event".into());
            }
            CGEvent::from_ptr(cg_event)
        };
        event.post(CGEventTapLocation::HID);
        Ok(())
    }

    extern "C" {
        fn CGEventCreateScrollWheelEvent(
            source: *const core_graphics::sys::CGEventSource,
            units: u32,
            wheel_count: u32,
            wheel1: i32,
            ...
        ) -> *mut core_graphics::sys::CGEvent;
    }

    /// Convert a key name to a macOS virtual key code.
    fn key_name_to_code(name: &str) -> CGKeyCode {
        match name.to_lowercase().as_str() {
            "return" | "enter" => 0x24,
            "tab" => 0x30,
            "space" => 0x31,
            "delete" | "backspace" => 0x33,
            "escape" | "esc" => 0x35,
            "left" => 0x7B,
            "right" => 0x7C,
            "down" => 0x7D,
            "up" => 0x7E,
            "a" => 0x00, "s" => 0x01, "d" => 0x02, "f" => 0x03,
            "h" => 0x04, "g" => 0x05, "z" => 0x06, "x" => 0x07,
            "c" => 0x08, "v" => 0x09, "b" => 0x0B, "q" => 0x0C,
            "w" => 0x0D, "e" => 0x0E, "r" => 0x0F, "y" => 0x10,
            "t" => 0x11, "o" => 0x1F, "u" => 0x20, "i" => 0x22,
            "p" => 0x23, "l" => 0x25, "j" => 0x26, "k" => 0x28,
            "n" => 0x2D, "m" => 0x2E,
            _ => 0x00,
        }
    }

    /// Convert modifier names to CGEventFlags.
    fn modifiers_to_flags(modifiers: &[String]) -> CGEventFlags {
        let mut flags = CGEventFlags::CGEventFlagNull;
        for m in modifiers {
            match m.to_lowercase().as_str() {
                "cmd" | "command" => flags |= CGEventFlags::CGEventFlagCommand,
                "shift" => flags |= CGEventFlags::CGEventFlagShift,
                "alt" | "option" => flags |= CGEventFlags::CGEventFlagAlternate,
                "ctrl" | "control" => flags |= CGEventFlags::CGEventFlagControl,
                _ => {}
            }
        }
        flags
    }
}

/// Execute a batch of actions sequentially with delays between them.
pub fn execute_batch(actions: &[Action]) -> Result<(), String> {
    for (i, action) in actions.iter().enumerate() {
        execute(action)?;
        // Delay between actions (except after the last one)
        if i + 1 < actions.len() && action.action_type != "wait" && action.action_type != "noop" {
            // Hotkeys (e.g. Cmd+Space) trigger UI transitions that need time to render
            let delay = match action.action_type.as_str() {
                "hotkey" | "osascript" => 1000,
                "keypress" => 300,
                _ => 150,
            };
            std::thread::sleep(std::time::Duration::from_millis(delay));
        }
    }
    Ok(())
}

/// Execute an action on the local machine.
pub fn execute(action: &Action) -> Result<(), String> {
    log::info!("Executing action: {:?}", action.action_type);

    #[cfg(target_os = "macos")]
    {
        match action.action_type.as_str() {
            "click" => {
                let coords = action
                    .coordinates
                    .as_ref()
                    .ok_or("click requires coordinates")?;
                if coords.len() < 2 {
                    return Err("click requires [x, y]".into());
                }
                macos::click(coords[0], coords[1])
            }
            "type" => {
                let text = action.text.as_deref().ok_or("type requires text")?;
                macos::type_text(text)
            }
            "keypress" | "hotkey" => {
                let key = action.key.as_deref().ok_or("keypress requires key")?;
                let mods = action.modifiers.as_deref().unwrap_or(&[]);
                macos::keypress(key, mods)
            }
            "scroll" => {
                let default_coords = vec![0.0, 100.0];
                let coords = action.coordinates.as_ref().unwrap_or(&default_coords);
                let delta = coords.get(1).copied().unwrap_or(100.0) as i32;
                macos::scroll(delta)
            }
            "osascript" => {
                let script = action.text.as_deref().ok_or("osascript requires text")?;
                log::info!("Running osascript: {}", &script[..script.len().min(200)]);
                let output = std::process::Command::new("osascript")
                    .arg("-e")
                    .arg(script)
                    .output()
                    .map_err(|e| format!("osascript failed to launch: {}", e))?;
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    log::warn!("osascript error: {}", stderr);
                    // Don't fail the batch — log and continue
                }
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.is_empty() {
                    log::info!("osascript output: {}", stdout.trim());
                }
                Ok(())
            }
            "wait" | "noop" => Ok(()),
            other => Err(format!("Unknown action type: {}", other)),
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(format!(
            "Action execution not supported on this platform: {}",
            action.action_type
        ))
    }
}
