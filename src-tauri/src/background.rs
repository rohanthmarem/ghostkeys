use crate::{engine, TypingStatus};
use serde_json::{json, Value};

fn ensure_idle() -> Result<(), String> {
    if engine().is_running()
        || matches!(
            engine().get_status(),
            TypingStatus::Typing | TypingStatus::Countdown | TypingStatus::Paused
        )
    {
        Err("Stop typing before changing the destination".into())
    } else {
        Ok(())
    }
}

#[tauri::command]
pub fn get_background_target() -> Value {
    #[cfg(target_os = "macos")]
    {
        mac::target_info()
    }
    #[cfg(not(target_os = "macos"))]
    {
        json!(null)
    }
}

#[tauri::command]
pub async fn capture_background_target() -> Result<Value, String> {
    ensure_idle()?;
    crate::platform::ensure_keyboard_access()?;
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    capture_target(None)
}

pub fn capture_target(pid: Option<i32>) -> Result<Value, String> {
    ensure_idle()?;
    crate::platform::ensure_keyboard_access()?;
    #[cfg(target_os = "macos")]
    {
        mac::capture(pid)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = pid;
        Err("Background typing is available on macOS".into())
    }
}

#[tauri::command]
pub fn clear_background_target() -> Result<(), String> {
    ensure_idle()?;
    #[cfg(target_os = "macos")]
    {
        *mac::TARGET.lock() = None;
    }
    Ok(())
}

#[cfg(target_os = "macos")]
pub mod mac {
    use super::*;
    use core_foundation::{
        array::CFArray,
        base::{CFType, CFTypeRef, TCFType},
        boolean::CFBoolean,
        string::{CFString, CFStringRef},
    };
    use core_graphics::{
        event::{CGEvent, CGEventFlags},
        event_source::{CGEventSource, CGEventSourceStateID},
    };
    use once_cell::sync::Lazy;
    use parking_lot::Mutex;
    use std::{ffi::c_void, ptr, thread, time::Duration};

    type AXElement = *const c_void;
    #[repr(C)]
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct TextRange {
        location: isize,
        length: isize,
    }
    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn AXUIElementCreateSystemWide() -> AXElement;
        fn AXUIElementCreateApplication(pid: i32) -> AXElement;
        fn AXUIElementCopyAttributeValue(
            element: AXElement,
            attribute: CFStringRef,
            value: *mut CFTypeRef,
        ) -> i32;
        fn AXUIElementIsAttributeSettable(
            element: AXElement,
            attribute: CFStringRef,
            settable: *mut u8,
        ) -> i32;
        fn AXUIElementSetAttributeValue(
            element: AXElement,
            attribute: CFStringRef,
            value: CFTypeRef,
        ) -> i32;
        fn AXUIElementGetPid(element: AXElement, pid: *mut i32) -> i32;
        fn AXValueGetValue(value: CFTypeRef, kind: u32, result: *mut c_void) -> u8;
    }

    #[derive(Clone)]
    pub struct Element(CFType);
    // AXUIElement references are retained CF objects; calls are supported from worker threads.
    unsafe impl Send for Element {}
    unsafe impl Sync for Element {}
    impl Element {
        fn enable_accessibility(&self) {
            let value = CFBoolean::true_value();
            for name in ["AXManualAccessibility", "AXEnhancedUserInterface"] {
                let name = CFString::new(name);
                unsafe {
                    AXUIElementSetAttributeValue(
                        self.raw(),
                        name.as_concrete_TypeRef(),
                        value.as_CFTypeRef(),
                    );
                }
            }
        }
        fn focused_editor(&self, budget: &mut usize) -> Option<Element> {
            if *budget == 0 {
                return None;
            }
            *budget -= 1;
            let focused = self
                .attribute("AXFocused")
                .ok()
                .and_then(|v| v.0.downcast::<CFBoolean>())
                .map(bool::from)
                .unwrap_or(false);
            if focused
                && matches!(
                    self.text("AXRole").ok().as_deref(),
                    Some("AXTextArea" | "AXTextField" | "AXComboBox")
                )
            {
                return Some(self.clone());
            }
            let children = self.attribute("AXChildren").ok()?.0.downcast::<CFArray>()?;
            for child in children.iter() {
                let child = Element(unsafe { CFType::wrap_under_get_rule(*child) });
                if let Some(editor) = child.focused_editor(budget) {
                    return Some(editor);
                }
            }
            None
        }
        fn raw(&self) -> AXElement {
            self.0.as_CFTypeRef()
        }
        fn attribute(&self, name: &str) -> Result<Element, String> {
            let name = CFString::new(name);
            let mut value = ptr::null();
            let error = unsafe {
                AXUIElementCopyAttributeValue(self.raw(), name.as_concrete_TypeRef(), &mut value)
            };
            if error != 0 || value.is_null() {
                return Err(format!(
                    "The editor does not expose {name} (macOS error {error})"
                ));
            }
            Ok(Element(unsafe { CFType::wrap_under_create_rule(value) }))
        }
        fn text(&self, name: &str) -> Result<String, String> {
            self.attribute(name)?
                .0
                .downcast::<CFString>()
                .map(|s| s.to_string())
                .ok_or_else(|| format!("The editor does not expose {name} as text"))
        }
        fn can_set(&self, name: &str) -> bool {
            let name = CFString::new(name);
            let mut settable = 0;
            unsafe {
                AXUIElementIsAttributeSettable(
                    self.raw(),
                    name.as_concrete_TypeRef(),
                    &mut settable,
                ) == 0
                    && settable != 0
            }
        }
        fn selection(&self) -> Result<TextRange, String> {
            let value = self.attribute("AXSelectedTextRange")?;
            let mut range = TextRange {
                location: 0,
                length: 0,
            };
            if unsafe {
                AXValueGetValue(
                    value.0.as_CFTypeRef(),
                    4,
                    &mut range as *mut _ as *mut c_void,
                )
            } == 0
            {
                return Err("The editor does not expose its insertion point".into());
            }
            Ok(range)
        }
        fn set_text(&self, name: &str, text: &str) -> Result<(), String> {
            let name = CFString::new(name);
            let text = CFString::new(text);
            let error = unsafe {
                AXUIElementSetAttributeValue(
                    self.raw(),
                    name.as_concrete_TypeRef(),
                    text.as_CFTypeRef(),
                )
            };
            if error == 0 {
                Ok(())
            } else {
                Err(format!(
                    "The editor refused the background edit (macOS error {error})"
                ))
            }
        }
    }

    #[derive(Clone)]
    pub struct Target {
        app: Element,
        editor: Element,
        window: Element,
        pid: i32,
        name: String,
        title: String,
        direct: bool,
    }
    pub static TARGET: Lazy<Mutex<Option<Target>>> = Lazy::new(|| Mutex::new(None));

    pub fn target_info() -> Value {
        let system =
            Element(unsafe { CFType::wrap_under_create_rule(AXUIElementCreateSystemWide()) });
        let foreground_pid = system.attribute("AXFocusedApplication").ok().map(|app| {
            let mut pid = 0;
            unsafe {
                AXUIElementGetPid(app.raw(), &mut pid);
            }
            pid
        });
        TARGET
            .lock()
            .as_ref()
            .map(|t| {
                json!({"app": t.name, "window": t.title, "pid":t.pid,
            "isForeground": foreground_pid == Some(t.pid),
            "method": if t.direct { "accessibility" } else { "appKeys" }})
            })
            .unwrap_or(Value::Null)
    }

    pub fn capture(pid: Option<i32>) -> Result<Value, String> {
        let app = if let Some(pid) = pid {
            if pid <= 0 {
                return Err("Invalid app process ID".into());
            }
            Element(unsafe { CFType::wrap_under_create_rule(AXUIElementCreateApplication(pid)) })
        } else {
            let system =
                Element(unsafe { CFType::wrap_under_create_rule(AXUIElementCreateSystemWide()) });
            system.attribute("AXFocusedApplication")?
        };
        let mut pid = 0;
        if unsafe { AXUIElementGetPid(app.raw(), &mut pid) } != 0
            || pid == std::process::id() as i32
        {
            return Err("Click the destination editor, rather than Ghostkeys, during the five-second countdown".into());
        }
        app.enable_accessibility();
        let focused = app.attribute("AXFocusedUIElement")?;
        let editor = if matches!(
            focused.text("AXRole").ok().as_deref(),
            Some("AXTextArea" | "AXTextField" | "AXComboBox")
        ) {
            focused
        } else {
            app.attribute("AXFocusedWindow")?
                .focused_editor(&mut 1500)
                .unwrap_or(focused)
        };
        let role = editor.text("AXRole")?;
        let subrole = editor.text("AXSubrole").unwrap_or_default();
        if !matches!(role.as_str(), "AXTextArea" | "AXTextField" | "AXComboBox")
            || subrole == "AXSecureTextField"
        {
            return Err(format!("Click inside an editable document or text field, then capture it again (selected item: {role})"));
        }
        // Require readable text so an ignored key cannot be reported as successfully typed.
        editor.text("AXValue")?;
        editor.selection()?;
        let window = editor
            .attribute("AXWindow")
            .or_else(|_| app.attribute("AXFocusedWindow"))?;
        let name = app.text("AXTitle").unwrap_or_else(|_| format!("App {pid}"));
        // Chromium advertises AXSelectedText as writable but can ignore those edits.
        // Send real keys to that browser process and observe the resulting text.
        let direct = editor.can_set("AXSelectedText")
            && !matches!(
                name.as_str(),
                "Arc" | "Helium" | "Google Chrome" | "Chromium"
            );
        let target = Target {
            name,
            title: window.text("AXTitle").unwrap_or_default(),
            app,
            editor,
            window,
            pid,
            direct,
        };
        *TARGET.lock() = Some(target);
        Ok(target_info())
    }

    pub struct BackgroundWriter {
        target: Target,
        source: CGEventSource,
        last_value: String,
        last_selection: TextRange,
    }
    impl BackgroundWriter {
        pub fn new() -> Result<Option<Self>, String> {
            let Some(target) = TARGET.lock().clone() else {
                return Ok(None);
            };
            let last_value = target.editor.text("AXValue")?;
            let last_selection = target.editor.selection()?;
            let source = CGEventSource::new(CGEventSourceStateID::CombinedSessionState)
                .map_err(|_| "Could not create background keyboard events")?;
            Ok(Some(Self {
                target,
                source,
                last_value,
                last_selection,
            }))
        }
        fn validate(&self) -> Result<(), String> {
            crate::platform::ensure_keyboard_access()?;
            if self.target.editor.text("AXValue")? != self.last_value {
                return Err("The destination text changed outside Ghostkeys. Capture the editor again before continuing.".into());
            }
            if self.target.editor.selection()? != self.last_selection {
                return Err("The destination caret moved outside Ghostkeys. Capture the editor again before continuing.".into());
            }
            if !self.target.direct {
                let window = self.target.app.attribute("AXFocusedWindow")?;
                let focused = self.target.app.attribute("AXFocusedUIElement")?;
                let editor = if matches!(
                    focused.text("AXRole").ok().as_deref(),
                    Some("AXTextArea" | "AXTextField" | "AXComboBox")
                ) {
                    focused
                } else {
                    window.focused_editor(&mut 1500).unwrap_or(focused)
                };
                if window.0 != self.target.window.0 || editor.0 != self.target.editor.0 {
                    return Err(
                        "The destination window or browser tab changed. Capture the editor again."
                            .into(),
                    );
                }
            }
            Ok(())
        }
        fn key(&self, code: u16, text: Option<&str>) -> Result<(), String> {
            let directions: &[bool] = if text.is_some() {
                &[true]
            } else {
                &[true, false]
            };
            for &down in directions {
                let event = CGEvent::new_keyboard_event(self.source.clone(), code, down)
                    .map_err(|_| "Could not create a background key event")?;
                event.set_flags(CGEventFlags::empty());
                if let Some(text) = text {
                    event.set_string(text);
                }
                event.post_to_pid(self.target.pid);
            }
            Ok(())
        }
        fn confirm_change(&mut self) -> Result<(), String> {
            for _ in 0..20 {
                thread::sleep(Duration::from_millis(25));
                let value = self.target.editor.text("AXValue")?;
                let selection = self.target.editor.selection()?;
                if value != self.last_value || selection != self.last_selection {
                    self.last_value = value;
                    self.last_selection = selection;
                    return Ok(());
                }
            }
            Err("This editor did not accept background typing. No keys were sent to your active app. Use focused-window mode for this editor.".into())
        }
        pub fn type_char(&mut self, c: char) -> Result<(), String> {
            self.validate()?;
            if self.target.direct {
                self.target
                    .editor
                    .set_text("AXSelectedText", &c.to_string())?;
            } else {
                match c {
                    '\n' | '\r' => self.key(36, None)?,
                    '\t' => return Err("Tabs are not supported in background browser mode".into()),
                    _ => self.key(0, Some(&c.to_string()))?,
                }
            }
            self.confirm_change()
        }
        pub fn backspace(&mut self) -> Result<(), String> {
            self.validate()?;
            // Deliver Backspace only to the chosen process, never to the active app.
            self.key(51, None)?;
            self.confirm_change()
        }
    }
}
