//! macOS requires the user to allow keyboard control in System Settings.

#[cfg(target_os = "macos")]
#[link(name = "ApplicationServices", kind = "framework")]
extern "C" {
    fn AXIsProcessTrusted() -> bool;
}

pub fn accessibility_granted() -> bool {
    #[cfg(target_os = "macos")]
    {
        unsafe { AXIsProcessTrusted() }
    }
    #[cfg(not(target_os = "macos"))]
    {
        true
    }
}

pub fn ensure_keyboard_access() -> Result<(), String> {
    if accessibility_granted() {
        Ok(())
    } else {
        Err("Allow ghostkeys in System Settings → Privacy & Security → Accessibility (Device Control and Data Access on newer macOS), then try again.".into())
    }
}

#[tauri::command]
pub fn get_platform_status() -> serde_json::Value {
    serde_json::json!({
        "isMac": cfg!(target_os = "macos"),
        "accessibilityGranted": accessibility_granted(),
        "shortcut": if cfg!(target_os = "macos") { "Control+Option+S" } else { "Ctrl+Alt+S" },
    })
}

#[tauri::command]
pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("/usr/bin/open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .status()
            .map_err(|e| format!("Could not open System Settings: {e}"))?;
        if !status.success() {
            return Err("Could not open System Settings".into());
        }
    }
    Ok(())
}
