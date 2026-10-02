use crate::{engine, Config};
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};
use tauri_plugin_store::StoreExt;

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub name: String,
    pub config: Config,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Preferences {
    pub auto_pause: bool,
    pub remember_draft: bool,
    pub start_shortcut: String,
    pub pause_shortcut: String,
    pub presets: Vec<Preset>,
}
impl Default for Preferences {
    fn default() -> Self {
        Self {
            auto_pause: true,
            remember_draft: true,
            start_shortcut: "Control+Alt+S".into(),
            pause_shortcut: "Control+Alt+P".into(),
            presets: vec![],
        }
    }
}
impl Preferences {
    pub fn shortcuts(&self) -> Result<[Shortcut; 2], String> {
        let parse = |text: &str| -> Result<Shortcut, String> {
            let shortcut: Shortcut = text.parse().map_err(|e| format!("Invalid shortcut: {e}"))?;
            if shortcut.mods.is_empty() {
                return Err("Shortcuts need at least one modifier key.".into());
            }
            Ok(shortcut)
        };
        let keys = [parse(&self.start_shortcut)?, parse(&self.pause_shortcut)?];
        if keys[0] == keys[1] {
            return Err("Choose different shortcuts for start and pause.".into());
        }
        Ok(keys)
    }
    pub fn validate(&self) -> Result<(), String> {
        self.shortcuts()?;
        if self.presets.len() > 30 {
            return Err("You can save up to 30 presets.".into());
        }
        let mut names = std::collections::HashSet::new();
        for preset in &self.presets {
            if preset.name.trim().is_empty()
                || preset.name.len() > 80
                || !names.insert(preset.name.trim().to_lowercase())
            {
                return Err("Give each preset a unique name (up to 80 characters).".into());
            }
            preset.config.validate()?;
        }
        Ok(())
    }
}
pub struct PreferenceState(pub Mutex<Preferences>);
pub struct ShortcutRecording(pub std::sync::atomic::AtomicBool);
#[tauri::command]
pub fn set_shortcut_recording(app: AppHandle, recording: bool) {
    app.state::<ShortcutRecording>()
        .0
        .store(recording, std::sync::atomic::Ordering::SeqCst);
}

#[tauri::command]
pub fn get_preferences(app: AppHandle) -> Preferences {
    app.state::<PreferenceState>().0.lock().clone()
}

#[tauri::command]
pub fn set_preferences(app: AppHandle, preferences: Preferences) -> Result<(), String> {
    if engine().is_running() {
        return Err("Stop typing before changing preferences.".into());
    }
    preferences.validate()?;
    let state = app.state::<PreferenceState>();
    let mut current = state.0.lock();
    let old_keys = current.shortcuts()?;
    let new_keys = preferences.shortcuts()?;
    let shortcuts = app.global_shortcut();
    let mut added = Vec::new();
    for key in new_keys.iter().filter(|key| !old_keys.contains(key)) {
        if let Err(e) = shortcuts.register(*key) {
            for key in added {
                let _ = shortcuts.unregister(key);
            }
            return Err(format!("Shortcut unavailable: {e}"));
        }
        added.push(*key);
    }
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    let old_draft = store.get("draft");
    store.set("preferences", serde_json::to_value(&preferences).unwrap());
    if preferences.remember_draft {
        store.set("draft", serde_json::json!({"content": engine().get_content(), "name": engine().get_file_name()}));
    } else {
        store.delete("draft");
    }
    if let Err(e) = store.save() {
        store.set("preferences", serde_json::to_value(&*current).unwrap());
        if let Some(draft) = old_draft {
            store.set("draft", draft);
        } else {
            store.delete("draft");
        }
        for key in added {
            let _ = shortcuts.unregister(key);
        }
        return Err(format!("Could not save preferences: {e}"));
    }
    for key in old_keys.iter().filter(|key| !new_keys.contains(key)) {
        let _ = shortcuts.unregister(*key);
    }
    engine().set_auto_pause(preferences.auto_pause);
    *current = preferences;
    Ok(())
}

pub fn save_draft(app: &AppHandle) -> Result<(), String> {
    if !app.state::<PreferenceState>().0.lock().remember_draft {
        return Ok(());
    }
    let store = app.store("settings.json").map_err(|e| e.to_string())?;
    store.set(
        "draft",
        serde_json::json!({"content": engine().get_content(), "name": engine().get_file_name()}),
    );
    store
        .save()
        .map_err(|e| format!("Could not save draft: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shortcut_validation_rejects_collisions_and_bare_keys() {
        let mut prefs = Preferences::default();
        assert!(prefs.validate().is_ok());
        prefs.pause_shortcut = prefs.start_shortcut.clone();
        assert!(prefs.validate().is_err());
        prefs.pause_shortcut = "P".into();
        assert!(prefs.validate().is_err());
    }
    #[test]
    fn presets_roundtrip_and_validate() {
        let mut prefs = Preferences::default();
        prefs.presets.push(Preset {
            name: "My preset".into(),
            config: Config::default(),
        });
        let saved = serde_json::to_string(&prefs).unwrap();
        let loaded: Preferences = serde_json::from_str(&saved).unwrap();
        assert_eq!(loaded.presets[0].name, "My preset");
        assert!(loaded.validate().is_ok());
        prefs.presets.push(prefs.presets[0].clone());
        assert!(prefs.validate().is_err());
    }
}
