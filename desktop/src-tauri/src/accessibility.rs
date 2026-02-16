//! macOS Accessibility Tree reader.
//!
//! Uses the AXUIElement API via raw FFI to walk the accessibility tree
//! of the frontmost application and serialize it to JSON.

#[cfg(not(target_os = "macos"))]
use serde_json::{json, Value};

#[cfg(target_os = "macos")]
mod macos {
    use core_foundation::base::{CFRelease, CFRetain, TCFType};
    use core_foundation::string::{CFString, CFStringRef};
    use serde_json::{json, Value};
    use std::ffi::c_void;

    // Raw AX FFI bindings
    type AXUIElementRef = *const c_void;
    type AXError = i32;
    const K_AX_ERROR_SUCCESS: AXError = 0;

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateApplication(pid: i32) -> AXUIElementRef;
        fn AXUIElementCopyAttributeValue(
            element: AXUIElementRef,
            attribute: CFStringRef,
            value: *mut *const c_void,
        ) -> AXError;
        fn AXIsProcessTrusted() -> bool;
    }

    fn get_string_attr(element: AXUIElementRef, attr: &str) -> Option<String> {
        let cf_attr = CFString::new(attr);
        let mut value: *const c_void = std::ptr::null();
        let err = unsafe {
            AXUIElementCopyAttributeValue(element, cf_attr.as_concrete_TypeRef(), &mut value)
        };
        if err != K_AX_ERROR_SUCCESS || value.is_null() {
            return None;
        }
        // Try to interpret as CFString
        let cf_str = unsafe { CFString::wrap_under_create_rule(value as CFStringRef) };
        Some(cf_str.to_string())
    }

    fn get_children(element: AXUIElementRef) -> Vec<AXUIElementRef> {
        let cf_attr = CFString::new("AXChildren");
        let mut value: *const c_void = std::ptr::null();
        let err = unsafe {
            AXUIElementCopyAttributeValue(element, cf_attr.as_concrete_TypeRef(), &mut value)
        };
        if err != K_AX_ERROR_SUCCESS || value.is_null() {
            return vec![];
        }

        extern "C" {
            fn CFArrayGetCount(array: *const c_void) -> isize;
            fn CFArrayGetValueAtIndex(array: *const c_void, idx: isize) -> *const c_void;
        }

        let count = unsafe { CFArrayGetCount(value) };
        let mut children = Vec::with_capacity(count as usize);
        for i in 0..count {
            let child = unsafe { CFArrayGetValueAtIndex(value, i) };
            if !child.is_null() {
                // CFArrayGetValueAtIndex returns a non-owning reference.
                // Retain each child so it survives the array release.
                unsafe { CFRetain(child) };
                children.push(child as AXUIElementRef);
            }
        }
        // Now safe to release the array — children are independently retained.
        unsafe { CFRelease(value) };
        children
    }

    fn walk_element(element: AXUIElementRef, depth: usize) -> Value {
        if depth > 8 {
            return json!({"truncated": true});
        }

        let role = get_string_attr(element, "AXRole").unwrap_or_default();
        let title = get_string_attr(element, "AXTitle").unwrap_or_default();
        let description = get_string_attr(element, "AXDescription").unwrap_or_default();
        let value_str = get_string_attr(element, "AXValue").unwrap_or_default();

        let child_refs = get_children(element);
        let children: Vec<Value> = child_refs
            .iter()
            .map(|c| {
                let result = walk_element(*c, depth + 1);
                // Release the retained child now that we're done with it.
                unsafe { CFRelease(*c) };
                result
            })
            .collect();

        let mut node = json!({
            "role": role,
            "title": title,
        });

        if !description.is_empty() {
            node["description"] = json!(description);
        }
        if !value_str.is_empty() {
            node["value"] = json!(value_str);
        }
        if !children.is_empty() {
            node["children"] = json!(children);
        }
        node
    }

    /// Read the accessibility tree of the application with the given PID.
    pub fn read_accessibility_tree(pid: i32) -> Value {
        let app = unsafe { AXUIElementCreateApplication(pid) };
        if app.is_null() {
            return json!({"error": "Could not create AXUIElement for app"});
        }
        let tree = walk_element(app, 0);
        unsafe { CFRelease(app as *const c_void) };
        tree
    }

    /// Read the accessibility tree of the frontmost application.
    pub fn read_frontmost_tree() -> Value {
        // Check accessibility permission first
        if !unsafe { AXIsProcessTrusted() } {
            return json!({"error": "Accessibility permission not granted"});
        }

        // Use osascript to get the PID of the frontmost app.
        let output = std::process::Command::new("osascript")
            .arg("-e")
            .arg("tell application \"System Events\" to unix id of first process whose frontmost is true")
            .output();

        match output {
            Ok(o) if o.status.success() => {
                let pid_str = String::from_utf8_lossy(&o.stdout).trim().to_string();
                match pid_str.parse::<i32>() {
                    Ok(pid) => read_accessibility_tree(pid),
                    Err(_) => json!({"error": "Could not parse frontmost PID"}),
                }
            }
            Ok(o) => {
                let stderr = String::from_utf8_lossy(&o.stderr).trim().to_string();
                json!({"error": format!("osascript failed: {}", stderr)})
            }
            Err(e) => json!({"error": format!("osascript failed: {}", e)}),
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::read_frontmost_tree;

#[cfg(not(target_os = "macos"))]
pub fn read_frontmost_tree() -> Value {
    json!({"error": "Accessibility tree is only supported on macOS"})
}
