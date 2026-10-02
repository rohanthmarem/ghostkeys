import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import type { Config } from "../lib/types";
import type { Preferences } from "../hooks/usePreferences";
import { Icon, shortcutLabel } from "./Icon";
import { sameConfig } from "./SettingsPanel";

function ShortcutInput({ label, value, onChange, disabled }: { label: string; value: string; onChange: (key: string) => void; disabled: boolean }) {
  const [recording, setRecording] = useState(false);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => {
    const cancelRecording = () => { input.current?.blur(); setRecording(false); };
    window.addEventListener("blur", cancelRecording);
    return () => {
      window.removeEventListener("blur", cancelRecording);
      void invoke("set_shortcut_recording", { recording: false });
    };
  }, []);
  return <label className="shortcut-row"><span>{label}</span><input ref={input} aria-label={label} value={recording ? "Press shortcut…" : shortcutLabel(value)} readOnly disabled={disabled}
    onFocus={() => { setRecording(true); void invoke("set_shortcut_recording", { recording: true }); }}
    onBlur={() => { setRecording(false); void invoke("set_shortcut_recording", { recording: false }); }}
    title="Click to record a shortcut" onKeyDown={e => {
      if (e.key === "Tab" || e.key === "Escape") return;
      e.preventDefault();
      if (!["Control", "Alt", "Meta", "Shift"].includes(e.key) && (e.ctrlKey || e.altKey || e.metaKey)) {
        const key = e.code.startsWith("Key") ? e.code.slice(3) : e.code.startsWith("Digit") ? e.code.slice(5) : e.code;
        onChange([e.ctrlKey && "Control", e.altKey && "Alt", e.metaKey && "Super", e.shiftKey && "Shift", key].filter(Boolean).join("+"));
        e.currentTarget.blur();
      }
    }} /></label>;
}
interface Props {
  preferences: Preferences; update: (p: Partial<Preferences>) => Promise<boolean>; config: Config;
  onApply: (c: Config) => void; disabled: boolean; section: "presets" | "app";
}
export function PreferencesPanel({ preferences, update, config, onApply, disabled, section }: Props) {
  const [name, setName] = useState("");
  const [keys, setKeys] = useState({ startShortcut: preferences.startShortcut, pauseShortcut: preferences.pauseShortcut });
  useEffect(() => { setKeys({ startShortcut: preferences.startShortcut, pauseShortcut: preferences.pauseShortcut }); }, [preferences.startShortcut, preferences.pauseShortcut]);
  const changed = keys.startShortcut !== preferences.startShortcut || keys.pauseShortcut !== preferences.pauseShortcut;
  if (section === "presets") return <div className="settings-content">
    <form className="preset-form" onSubmit={async e => { e.preventDefault(); if (await update({ presets: [...preferences.presets, { name: name.trim(), config: { ...config } }] })) setName(""); }}>
      <label htmlFor="preset-name">Save current typing settings</label>
      <div><input id="preset-name" placeholder="Name your preset" value={name} maxLength={80} disabled={disabled} onChange={e => setName(e.target.value)} /><button className="button primary" disabled={disabled || !name.trim()}>Save</button></div>
    </form>
    {preferences.presets.length === 0 ? <p className="empty-presets">Your saved presets will appear here.</p> : <ul className="saved-presets">{preferences.presets.map(p => <li key={p.name}>
      <button className="preset-apply" disabled={disabled} onClick={() => onApply(p.config)}><span><strong>{p.name}</strong><small>{p.config.baseWpm} WPM · {p.config.countdownSeconds}s delay</small></span>{sameConfig(p.config, config) ? <Icon name="check" /> : <Icon name="arrow" />}</button>
      <button className="icon-button" disabled={disabled} aria-label={`Delete preset ${p.name}`} title="Delete preset" onClick={() => void update({ presets: preferences.presets.filter(item => item.name !== p.name) })}><Icon name="trash" size={16} /></button>
    </li>)}</ul>}
  </div>;
  return <div className="settings-content">
    <div className="settings-group">
      <label className="switch-row"><span>Pause when switching apps</span><input className="switch" type="checkbox" checked={preferences.autoPause} disabled={disabled} onChange={e => void update({ autoPause: e.target.checked })} /></label>
      <label className="switch-row"><span>Remember draft after quitting</span><input className="switch" type="checkbox" checked={preferences.rememberDraft} disabled={disabled} onChange={e => void update({ rememberDraft: e.target.checked })} /></label>
    </div>
    <div className="settings-group"><h3>Global shortcuts</h3>
      <ShortcutInput label="Start / stop" value={keys.startShortcut} disabled={disabled} onChange={startShortcut => setKeys(k => ({ ...k, startShortcut }))} />
      <ShortcutInput label="Pause / resume" value={keys.pauseShortcut} disabled={disabled} onChange={pauseShortcut => setKeys(k => ({ ...k, pauseShortcut }))} />
      {changed && <div className="shortcut-actions"><button className="text-button" disabled={disabled} onClick={() => setKeys({ startShortcut: preferences.startShortcut, pauseShortcut: preferences.pauseShortcut })}>Cancel</button><button className="button primary" disabled={disabled} onClick={() => void update(keys)}>Apply shortcuts</button></div>}
    </div>
  </div>;
}
