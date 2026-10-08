//! Optional loopback control API. Disabled unless GHOSTKEYS_API_TOKEN is set.
use ghostkeys_lib::{engine, Config, TypingStatus};
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter};

fn execute(app: &AppHandle, value: Value) -> Result<Value, String> {
    let action = value["action"].as_str().ok_or("Missing action")?;
    let eng = engine();
    let active = eng.is_running() || matches!(eng.get_status(), TypingStatus::Typing | TypingStatus::Countdown | TypingStatus::Paused);
    match action {
        "status" => Ok(json!({"status": eng.get_status(), "progress": eng.get_progress(), "fileName": eng.get_file_name(), "config": eng.get_config(), "backgroundTarget": ghostkeys_lib::background::get_background_target()})),
        "capture_target" => ghostkeys_lib::background::capture_target(value["pid"].as_i64().map(|pid| pid as i32)),
        "clear_target" => { ghostkeys_lib::background::clear_background_target()?; Ok(json!({"status":"cleared"})) },
        "load" => {
            if active { return Err("Stop the current typing session first".into()); }
            let content = value["text"].as_str().ok_or("Missing text")?;
            if content.is_empty() || content.len() > 100_000 { return Err("Text must contain 1 to 100000 bytes".into()); }
            eng.set_content(content.to_owned(), "MCP text".into());
            eng.set_status(TypingStatus::Ready, app);
            let _ = app.emit("content-loaded", json!({"content":content, "fileName":"MCP text"}));
            Ok(json!({"status":"ready","characters": content.chars().count()}))
        },
        "configure" => {
            if active { return Err("Stop the current typing session first".into()); }
            let config: Config = serde_json::from_value(value["config"].clone()).map_err(|e| e.to_string())?;
            if !(5..=300).contains(&config.base_wpm) || config.countdown_seconds > 30 || !config.wpm_variance.is_finite() || !(0.0..=1.0).contains(&config.wpm_variance) || !(0.0..=0.15).contains(&config.mistake_rate) || !(0.0..=1.0).contains(&config.correction_rate) || !(0.0..=1.0).contains(&config.thinking_pause_chance) || config.punctuation_pause > 10000 || config.paragraph_pause > 10000 || config.thinking_pause_duration > 30000 { return Err("Configuration is outside supported limits".into()); }
            eng.set_config(config);
            let _ = app.emit("config-changed", eng.get_config());
            Ok(json!({"status":"configured"}))
        },
        "start" => {
            ghostkeys_lib::platform::ensure_keyboard_access()?;
            if active { return Err("A typing session is already active".into()); }
            if eng.get_progress().total == 0 { return Err("Load text first".into()); }
            let app = app.clone();
            // Reserve the session before returning, so repeated starts cannot race.
            eng.set_status(TypingStatus::Countdown, &app);
            let eng = eng.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(error) = eng.clone().run(app.clone()).await {
                    eprintln!("Typing error: {error}");
                    eng.set_status(TypingStatus::Error, &app);
                    let _ = app.emit("typing-error", json!({"message":error}));
                }
            });
            Ok(json!({"status":"countdown"}))
        },
        "stop" => { eng.stop(); Ok(json!({"status":"stopping"})) },
        "pause" => {
            if eng.get_status() != TypingStatus::Typing { return Err("Typing is not running".into()); }
            eng.pause(); eng.set_status(TypingStatus::Paused, app); Ok(json!({"status":"paused"}))
        },
        "resume" => {
            if eng.get_status() != TypingStatus::Paused { return Err("Typing is not paused".into()); }
            eng.resume(); eng.set_status(TypingStatus::Typing, app); Ok(json!({"status":"typing"}))
        },
        _ => Err("Unknown action".into()),
    }
}

pub fn start(app: AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let Ok(token) = std::env::var("GHOSTKEYS_API_TOKEN") else { return Ok(()); };
    if token.len() < 32 { return Err("GHOSTKEYS_API_TOKEN must have at least 32 characters".into()); }
    let server = tiny_http::Server::http("127.0.0.1:8789").map_err(|e| e.to_string())?;
    std::thread::spawn(move || {
        for mut req in server.incoming_requests() {
            let authorized = req.headers().iter().any(|h| h.field.equiv("Authorization") && h.value.as_str() == format!("Bearer {}", token));
            let (status, value) = if !authorized {
                (401, json!({"error":"Unauthorized"}))
            } else if req.method() != &tiny_http::Method::Post || req.url() != "/command" {
                (404, json!({"error":"Not found"}))
            } else if req.body_length().is_none_or(|n| n > 110000) {
                (413, json!({"error":"Request too large or missing Content-Length"}))
            } else {
                match serde_json::from_reader::<_, Value>(req.as_reader()) {
                    Ok(value) => match execute(&app, value) { Ok(result) => (200, result), Err(error) => (409, json!({"error":error})) },
                    Err(_) => (400, json!({"error":"Invalid JSON"})),
                }
            };
            let response = tiny_http::Response::from_string(value.to_string()).with_status_code(status)
                .with_header(tiny_http::Header::from_bytes("Content-Type", "application/json").unwrap());
            let _ = req.respond(response);
        }
    });
    Ok(())
}
