//! Platform helpers: frontmost app name, window bounds, etc.
//!
//! Uses AppleScript via `osascript` for reliable cross-version macOS support.

/// Get the name of the frontmost application.
#[cfg(target_os = "macos")]
pub fn frontmost_app() -> (String, i32) {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(concat!(
            "tell application \"System Events\"\n",
            "  set fp to first process whose frontmost is true\n",
            "  set appName to name of fp\n",
            "  set appPID to unix id of fp\n",
            "  return appName & \"|\" & (appPID as text)\n",
            "end tell"
        ))
        .output();

    match output {
        Ok(o) if o.status.success() => {
            let raw = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if let Some((name, pid_str)) = raw.rsplit_once('|') {
                let pid = pid_str.parse::<i32>().unwrap_or(-1);
                (name.to_string(), pid)
            } else {
                (raw, -1)
            }
        }
        _ => (String::new(), -1),
    }
}

#[cfg(not(target_os = "macos"))]
pub fn frontmost_app() -> (String, i32) {
    (String::new(), -1)
}

/// Get the bounds of the focused window as (x, y, width, height).
/// Falls back to the main display bounds if the focused window can't be read.
#[cfg(target_os = "macos")]
pub fn focused_window_bounds() -> (f64, f64, f64, f64) {
    let output = std::process::Command::new("osascript")
        .arg("-e")
        .arg(concat!(
            "tell application \"System Events\"\n",
            "  set frontProc to first process whose frontmost is true\n",
            "  try\n",
            "    tell frontProc\n",
            "      set {x, y} to position of window 1\n",
            "      set {w, h} to size of window 1\n",
            "      return (x as text) & \",\" & (y as text) & \",\" & (w as text) & \",\" & (h as text)\n",
            "    end tell\n",
            "  on error\n",
            "    return \"fail\"\n",
            "  end try\n",
            "end tell"
        ))
        .output();

    if let Ok(out) = output {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s != "fail" {
            let parts: Vec<&str> = s.split(',').collect();
            if parts.len() == 4 {
                if let (Ok(x), Ok(y), Ok(w), Ok(h)) = (
                    parts[0].trim().parse::<f64>(),
                    parts[1].trim().parse::<f64>(),
                    parts[2].trim().parse::<f64>(),
                    parts[3].trim().parse::<f64>(),
                ) {
                    return (x, y, w, h);
                }
            }
        }
    }

    // Fallback to full display
    let (sw, sh) = crate::capture::screen_size();
    (0.0, 0.0, sw, sh)
}

#[cfg(not(target_os = "macos"))]
pub fn focused_window_bounds() -> (f64, f64, f64, f64) {
    (0.0, 0.0, 1920.0, 1080.0)
}
