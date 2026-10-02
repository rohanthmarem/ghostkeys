import { useEffect, useRef, useState } from "react";
import type { Config } from "../lib/types";
import type { Preferences } from "../hooks/usePreferences";
import { SettingsPanel } from "./SettingsPanel";
import { PreferencesPanel } from "./PreferencesPanel";
import { Icon } from "./Icon";
export function SettingsDialog({ onClose, config, onUpdate, onReset, preferences, updatePreferences, disabled, error }: {
  onClose: () => void; config: Config; onUpdate: (c: Partial<Config>) => void; onReset: () => void;
  preferences: Preferences; updatePreferences: (p: Partial<Preferences>) => Promise<boolean>; disabled: boolean; error: string | null;
}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const tabs = ["typing", "presets", "app"] as const;
  const [tab, setTab] = useState<typeof tabs[number]>("typing");
  useEffect(() => { const el = dialog.current!; el.showModal(); return () => el.close(); }, []);
  return <dialog className="settings-dialog" ref={dialog} aria-labelledby="dialog-title" onCancel={onClose} onClick={e => { if (e.target === e.currentTarget) onClose(); }}>
    <div className="dialog-surface">
      <header className="dialog-header"><h2 id="dialog-title">Settings</h2><button className="icon-button" onClick={onClose} aria-label="Close settings" title="Close settings"><Icon name="close" /></button></header>
      <div className="dialog-tabs" role="tablist" aria-label="Settings sections">{tabs.map((name, index) => <button key={name} id={`tab-${name}`} role="tab" aria-selected={tab === name} aria-controls="settings-tabpanel" tabIndex={tab === name ? 0 : -1} onClick={() => setTab(name)} onKeyDown={e => {
        if (["ArrowLeft", "ArrowRight", "Home", "End"].includes(e.key)) {
          e.preventDefault(); const next = e.key === "Home" ? 0 : e.key === "End" ? 2 : (index + (e.key === "ArrowRight" ? 1 : 2)) % 3;
          setTab(tabs[next]); document.getElementById(`tab-${tabs[next]}`)?.focus();
        }
      }}>{name === "typing" ? "Typing" : name === "presets" ? "My presets" : "App"}</button>)}</div>
      <div className="dialog-scroll" id="settings-tabpanel" role="tabpanel" aria-labelledby={`tab-${tab}`} tabIndex={0} key={tab}>
        {error && <p className="notice error" role="alert">{error}</p>}
        {tab === "typing" ? <SettingsPanel config={config} onUpdate={onUpdate} onReset={onReset} disabled={disabled} /> : <PreferencesPanel section={tab} preferences={preferences} update={updatePreferences} config={config} onApply={onUpdate} disabled={disabled} />}
      </div>
      <footer className="dialog-footer"><button className="button secondary" onClick={onClose}>Done</button></footer>
    </div>
  </dialog>;
}
