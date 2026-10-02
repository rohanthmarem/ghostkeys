import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { ComposeEditor } from "./components/ComposeEditor";
import { CountdownOverlay } from "./components/CountdownOverlay";
import { SettingsDialog } from "./components/SettingsDialog";
import { Icon, Mark, shortcutLabel } from "./components/Icon";
import { sameConfig, typingStyles } from "./components/SettingsPanel";
import { useTypingState } from "./hooks/useTypingState";
import { useConfig } from "./hooks/useConfig";
import { usePreferences } from "./hooks/usePreferences";
import { usePlatformInfo } from "./hooks/usePlatformInfo";
import "./styles/globals.css";

export default function App() {
  const { state, countdown, changeContent, changeSelection, selectedCount, start, stop, pause, resume, loading, isSaving } = useTypingState();
  const settings = useConfig();
  const prefs = usePreferences();
  const { platformInfo, needsAccessibility, permissionError, openingSettings, openSettings } = usePlatformInfo();
  const [settingsOpen, setSettingsOpen] = useState(false);
  const [busy, setBusy] = useState(false);
  const [actionError, setActionError] = useState<string | null>(null);
  const [speed, setSpeed] = useState(String(settings.config.baseWpm));
  const active = ["typing", "countdown", "paused"].includes(state.status);
  const shortcut = shortcutLabel(prefs.preferences.startShortcut);
  const controlsDisabled = active || loading || settings.loading || prefs.loading || busy || prefs.saving;
  const unavailable = controlsDisabled || !platformInfo || needsAccessibility || !state.content?.trim() || isSaving || settings.saving || !!settings.error;
  const styles = [...typingStyles, ...prefs.preferences.presets];
  const styleIndex = styles.findIndex(p => sameConfig(p.config, settings.config));
  const error = actionError || prefs.error || settings.error || state.errorMessage || permissionError;
  const status = { idle: "Empty draft", ready: "Ready", countdown: "Starting", typing: "Typing", paused: "Paused", done: "Finished", error: "Needs attention" }[state.status];
  const runAction = useCallback(async (action: () => Promise<void>) => {
    setBusy(true); setActionError(null);
    try { await action(); } catch (e) { setActionError(String(e)); } finally { setBusy(false); }
  }, []);
  useEffect(() => { setSpeed(String(settings.config.baseWpm)); }, [settings.config.baseWpm]);
  useEffect(() => { if (active) setSettingsOpen(false); }, [active]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape" && active) { event.preventDefault(); void stop(); }
      if (!active && (event.metaKey || event.ctrlKey) && event.key === ",") { event.preventDefault(); setSettingsOpen(true); }
    };
    const onBlur = () => { void invoke("set_shortcut_recording", { recording: false }); };
    window.addEventListener("keydown", onKey); window.addEventListener("blur", onBlur);
    return () => { window.removeEventListener("keydown", onKey); window.removeEventListener("blur", onBlur); };
  }, [active, stop]);
  const commitSpeed = () => {
    const value = Math.max(20, Math.min(200, Math.round(Number(speed) || settings.config.baseWpm)));
    setSpeed(String(value));
    if (value !== settings.config.baseWpm) settings.updateConfig({ baseWpm: value });
  };
  return <div className="app-shell">
    <header className="app-header">
      <div className="brand"><Mark /><h1>ghostkeys</h1></div>
      <nav aria-label="App controls"><button className="icon-button" title="Mini window" aria-label="Mini window" onClick={() => void runAction(() => invoke("show_widget"))}><Icon name="mini" /></button><button className="icon-button" title="Settings (⌘,)" aria-label="Settings" disabled={active} onClick={() => setSettingsOpen(true)}><Icon name="settings" /></button></nav>
    </header>
    <main className="writing-surface">
      <ComposeEditor content={state.content ?? ""} fileName={state.fileName} disabled={active || loading || busy} onChange={changeContent} onSelection={changeSelection} selectedCount={selectedCount} />
    </main>
    <footer className="transport">
      {error && !settingsOpen && <p className="notice error" role="alert">{error}</p>}
      {needsAccessibility && <div className="access-notice"><span>Allow access to type in other apps.</span><button className="text-button" disabled={openingSettings} onClick={() => void openSettings()}>{openingSettings ? "Opening…" : "Enable access"}<Icon name="arrow" size={15} /></button></div>}
      <div className="transport-options">
        <label className="style-control"><span>Style</span><div className="select-wrap"><select aria-label="Typing style" value={styleIndex < 0 ? "custom" : String(styleIndex)} disabled={controlsDisabled} onChange={e => { const chosen = styles[Number(e.target.value)]; if (chosen) settings.updateConfig(chosen.config); }}>{styleIndex < 0 && <option value="custom">Custom</option>}<optgroup label="Built-in">{typingStyles.map((p, i) => <option value={i} key={p.name}>{p.name}</option>)}</optgroup>{prefs.preferences.presets.length > 0 && <optgroup label="My presets">{prefs.preferences.presets.map((p, i) => <option value={i + typingStyles.length} key={p.name}>{p.name}</option>)}</optgroup>}</select><Icon name="chevron" size={14} /></div></label>
        <label className="speed-control"><span>Speed</span><div><input type="number" aria-label="Typing speed" min={20} max={200} step={1} inputMode="numeric" value={speed} disabled={controlsDisabled} onChange={e => setSpeed(e.target.value)} onBlur={commitSpeed} onKeyDown={e => { if (e.key === "Enter") e.currentTarget.blur(); }} /><span>WPM</span></div></label>
        <button className="delay-control" disabled={active} title="Change start delay" onClick={() => setSettingsOpen(true)}><Icon name="clock" size={15} />{settings.config.countdownSeconds}s delay</button>
        <div className="session-actions">{active ? <>
          {state.status !== "countdown" && <button className="button secondary" disabled={busy} onClick={() => void runAction(state.status === "paused" ? resume : pause)}><Icon name={state.status === "paused" ? "play" : "pause"} size={15} />{state.status === "paused" ? "Resume" : "Pause"}</button>}
          <button className="button primary" onClick={() => void stop()}><Icon name="stop" size={15} />Stop</button>
        </> : <button className="button primary start-button" disabled={unavailable} onClick={() => void runAction(async () => { await settings.flush(); await start(); })}><Icon name="play" size={15} />{selectedCount > 0 ? "Type selection" : state.status === "done" ? "Type again" : "Start typing"}</button>}</div>
      </div>
      <div className="transport-status"><span className="status" aria-live="polite"><span className={`status-dot ${state.status === "typing" ? "running" : ""}`} />{status}{(active || state.status === "done") && <span className="progress-count">{state.currentChar.toLocaleString()} / {state.totalChars.toLocaleString()}</span>}</span><kbd title="Start / stop from any app">{shortcut}</kbd></div>
      {(active || state.status === "done") && <div className="session-progress" role="progressbar" aria-label="Typing progress" aria-valuenow={Math.round(state.percent)} aria-valuemin={0} aria-valuemax={100}><div style={{ width: `${state.percent}%` }} /></div>}
    </footer>
    {settingsOpen && <SettingsDialog onClose={() => setSettingsOpen(false)} config={settings.config} onUpdate={settings.updateConfig} onReset={settings.resetConfig} preferences={prefs.preferences} updatePreferences={prefs.update} disabled={controlsDisabled} error={error} />}
    <CountdownOverlay countdown={countdown} visible={state.status === "countdown"} onStop={() => void stop()} />
  </div>;
}
