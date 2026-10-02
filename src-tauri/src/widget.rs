use tauri::{AppHandle, Manager};

pub fn configure(app: &AppHandle) -> tauri::Result<()> {
    if let Some(widget) = app.get_webview_window("widget") {
        widget.set_focusable(false)?;
        widget.set_always_on_top(true)?;
        #[cfg(target_os = "macos")]
        {
            widget.set_visible_on_all_workspaces(true)?;
            let native_widget = widget.clone();
            widget.run_on_main_thread(move || {
                use objc2_app_kit::{NSWindow, NSWindowCollectionBehavior as Behavior};
                if let Ok(pointer) = native_widget.ns_window() {
                    // Tauri owns this NSWindow; borrow it only on the UI thread.
                    let window = unsafe { &*pointer.cast::<NSWindow>() };
                    // Join other apps' full-screen and Stage Manager spaces:
                    // https://developer.apple.com/documentation/appkit/nswindow/collectionbehavior-swift.struct/canjoinallapplications
                    {
                        let mut behavior = Behavior::CanJoinAllSpaces
                            | Behavior::FullScreenAuxiliary
                            | Behavior::IgnoresCycle;
                        if objc2::available!(macos = 13.0) {
                            behavior |= Behavior::CanJoinAllApplications;
                        }
                        window.setCollectionBehavior(behavior);
                        window.setHidesOnDeactivate(false);
                    }
                }
            })?;
        }
    }
    Ok(())
}

pub fn show_progress_widget(app: &AppHandle) -> tauri::Result<()> {
    configure(app)?;
    if let Some(widget) = app.get_webview_window("widget") {
        widget.show()?;
        #[cfg(target_os = "macos")]
        {
            let native_widget = widget.clone();
            widget.run_on_main_thread(move || {
                if let Ok(pointer) = native_widget.ns_window() {
                    // Raise without making ghostkeys the key window or active app.
                    let window = unsafe { &*pointer.cast::<objc2_app_kit::NSWindow>() };
                    window.orderFrontRegardless();
                }
            })?;
        }
    }
    Ok(())
}
