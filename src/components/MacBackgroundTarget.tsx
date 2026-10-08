import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface Target { app: string; window: string; method: "accessibility" | "appKeys" }

export function MacBackgroundTarget({ disabled, onCapturingChange }: { disabled: boolean; onCapturingChange: (capturing: boolean) => void }) {
  const [isMac, setIsMac] = useState(false);
  const [target, setTarget] = useState<Target | null>(null);
  const [capturing, setCapturing] = useState(false);
  const [remaining, setRemaining] = useState(0);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    void invoke<{ isMac: boolean }>("get_platform_status").then((s) => setIsMac(s.isMac));
    const read = () => { void invoke<Target | null>("get_background_target").then(setTarget); };
    read();
    const timer = window.setInterval(read, 2000);
    return () => window.clearInterval(timer);
  }, []);
  if (!isMac) return null;
  const capture = async () => {
    setCapturing(true); setRemaining(5); setError(null);
    onCapturingChange(true);
    const timer = window.setInterval(() => setRemaining((n) => Math.max(0, n - 1)), 1000);
    try { setTarget(await invoke<Target>("capture_background_target")); }
    catch (error) { setError(String(error)); }
    finally { window.clearInterval(timer); setCapturing(false); onCapturingChange(false); }
  };
  return (
    <section className="rounded-lg border border-ghost-700 bg-ghost-900 p-4 space-y-3">
      <h2 className="font-semibold text-ghost-100">Typing destination</h2>
      <p className="text-sm text-ghost-300 leading-relaxed">
        {target ? <>Background: <strong>{target.app}</strong>{target.window && <> · {target.window}</>}. You can use other apps while it types. Keep this document and browser tab open.</>
          : "Focused window. To type in the background, capture an editor below."}
      </p>
      {capturing && <p className="text-accent-warning" role="status">Click inside the destination editor now. Capturing in {remaining}…</p>}
      <div className="flex gap-2">
        <button disabled={disabled || capturing} onClick={capture} className="rounded-lg bg-accent-primary px-3 py-2 text-sm text-white disabled:opacity-50">{target ? "Change editor" : "Capture background editor"}</button>
        {target && <button disabled={disabled || capturing} onClick={async () => {
          try { await invoke("clear_background_target"); setTarget(null); setError(null); }
          catch (error) { setError(String(error)); }
        }} className="rounded-lg bg-ghost-700 px-3 py-2 text-sm disabled:opacity-50">Use focused window</button>}
      </div>
      {target && <p className="text-xs text-ghost-400">Background mode types exact text with generated mistakes disabled. Editors that reject background edits stop with an error.</p>}
      {error && <p role="alert" className="text-sm text-accent-error">{error}</p>}
    </section>
  );
}
