use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformInfo {
    platform: &'static str,
    accessibility_granted: bool,
    shortcut_label: &'static str,
}

pub fn info() -> PlatformInfo {
    PlatformInfo {
        platform: std::env::consts::OS,
        accessibility_granted: accessibility_granted(),
        shortcut_label: if cfg!(target_os = "macos") {
            "Control+Option+S"
        } else {
            "Ctrl+Alt+S"
        },
    }
}

pub fn ensure_accessibility() -> Result<(), String> {
    if accessibility_granted() {
        Ok(())
    } else {
        Err("Allow ghostkeys in System Settings → Privacy & Security → Accessibility, then try Start again.".into())
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use core_foundation::{
        base::TCFType,
        boolean::CFBoolean,
        dictionary::CFDictionary,
        string::{CFString, CFStringRef},
    };

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXIsProcessTrusted() -> bool;
        fn AXIsProcessTrustedWithOptions(
            options: core_foundation::dictionary::CFDictionaryRef,
        ) -> bool;
        static kAXTrustedCheckOptionPrompt: CFStringRef;
    }

    pub fn is_trusted(prompt: bool) -> bool {
        // These system APIs are thread-safe. The dictionary owns its values for the call.
        unsafe {
            if !prompt {
                return AXIsProcessTrusted();
            }
            let key = CFString::wrap_under_get_rule(kAXTrustedCheckOptionPrompt);
            let options = CFDictionary::from_CFType_pairs(&[(key, CFBoolean::true_value())]);
            AXIsProcessTrustedWithOptions(options.as_concrete_TypeRef())
        }
    }
}

pub fn accessibility_granted() -> bool {
    #[cfg(target_os = "macos")]
    return macos::is_trusted(false);
    #[cfg(not(target_os = "macos"))]
    true
}

pub fn request_accessibility() -> bool {
    #[cfg(target_os = "macos")]
    return macos::is_trusted(true);
    #[cfg(not(target_os = "macos"))]
    true
}

pub fn open_accessibility_settings() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let status = std::process::Command::new("/usr/bin/open")
            .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility")
            .status()
            .map_err(|e| format!("Could not open System Settings: {e}"))?;
        if !status.success() {
            return Err("Could not open Accessibility settings.".into());
        }
    }
    Ok(())
}

/// Read the focused application's PID through Accessibility. A missing target
/// fails closed; no window titles or document contents are read.
#[cfg(target_os = "macos")]
pub fn focused_application() -> Option<i32> {
    use core_foundation::{
        base::{CFRelease, CFTypeRef, TCFType},
        string::CFString,
    };
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> CFTypeRef;
        fn AXUIElementCopyAttributeValue(
            element: CFTypeRef,
            attribute: core_foundation::string::CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementGetPid(element: CFTypeRef, pid: *mut i32) -> i32;
    }
    unsafe {
        let system = AXUIElementCreateSystemWide();
        if system.is_null() {
            return None;
        }
        let attribute = CFString::new("AXFocusedApplication");
        let mut app = std::ptr::null();
        let result =
            AXUIElementCopyAttributeValue(system, attribute.as_concrete_TypeRef(), &mut app);
        CFRelease(system);
        if result != 0 || app.is_null() {
            return None;
        }
        let mut pid = 0;
        let result = AXUIElementGetPid(app, &mut pid);
        CFRelease(app);
        if result == 0 && pid > 0 {
            Some(pid)
        } else {
            None
        }
    }
}
#[cfg(not(target_os = "macos"))]
pub fn focused_application() -> Option<i32> {
    None
}
