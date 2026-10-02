import { useCallback, useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { Config } from "../lib/types";
export interface Preferences {
  autoPause: boolean; rememberDraft: boolean; startShortcut: string; pauseShortcut: string;
  presets: { name: string; config: Config }[];
}
const defaults: Preferences = { autoPause: true, rememberDraft: true, startShortcut: "Control+Alt+S", pauseShortcut: "Control+Alt+P", presets: [] };
export function usePreferences() {
  const [preferences, setPreferences] = useState(defaults);
  const [loading, setLoading] = useState(true);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => { void invoke<Preferences>("get_preferences").then(setPreferences).catch(e => setError(String(e))).finally(() => setLoading(false)); }, []);
  const update = useCallback(async (updates: Partial<Preferences>) => {
    const next = { ...preferences, ...updates };
    setSaving(true); setError(null);
    try { await invoke("set_preferences", { preferences: next }); setPreferences(next); return true; }
    catch (e) { setError(String(e)); return false; }
    finally { setSaving(false); }
  }, [preferences]);
  return { preferences, loading, saving, error, update };
}
