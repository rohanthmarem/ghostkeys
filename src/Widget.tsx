import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useTypingState } from "./hooks/useTypingState";
import { Icon } from "./components/Icon";
import "./styles/globals.css";

function formatElapsed(milliseconds: number) {
  const seconds = Math.floor(milliseconds / 1000);
  const hours = Math.floor(seconds / 3600);
  const minutes = Math.floor(seconds / 60) % 60;
  const tail = `${String(minutes).padStart(2, "0")}:${String(seconds % 60).padStart(2, "0")}`;
  return hours ? `${hours}:${tail}` : tail;
}

export default function Widget() {
  const { state, countdown, pause, resume, stop } = useTypingState();
  const [timing, setTiming] = useState<{ elapsedMs: number; remainingMs: number | null }>({ elapsedMs: 0, remainingMs: null });
  const [busy, setBusy] = useState(false);
  const active = ["typing", "countdown", "paused"].includes(state.status);
  const remaining = Math.max(0, state.totalChars - state.currentChar);
  const label = { idle: "Ready", ready: "Ready", countdown: `Starting in ${countdown || "…"}`, typing: "Typing", paused: "Paused", done: "Finished", error: "Needs attention" }[state.status];
  useEffect(() => {
    let disposed = false;
    let timer: ReturnType<typeof setTimeout>;
    const refresh = async () => {
      try {
        const value = await invoke<typeof timing>("get_session_timing");
        if (!disposed) setTiming(value);
      } catch { /* Keep the last confirmed time while the backend reconnects. */ }
      if (!disposed) timer = setTimeout(refresh, 500);
    };
    void refresh();
    return () => { disposed = true; clearTimeout(timer); };
  }, [state.status]);
  const togglePause = async () => {
    setBusy(true);
    try { await (state.status === "paused" ? resume() : pause()); }
    finally { setBusy(false); }
  };
  return <div className="mini-window">
    <header data-tauri-drag-region><span data-tauri-drag-region>ghostkeys</span><span className="mini-label" aria-live="polite">{label}</span><button className="icon-button" aria-label="Hide mini window" title="Hide mini window" onMouseDown={e => e.preventDefault()} onClick={() => void getCurrentWindow().hide()}><Icon name="close" size={13} /></button></header>
    <div className="mini-content">
      <div className="mini-status"><strong>{remaining.toLocaleString()} <span>characters left</span></strong></div>
      <div className="mini-actions" onMouseDown={e => e.preventDefault()}>
        {active ? <><button className="mini-toggle" disabled={busy || state.status === "countdown"} aria-label={state.status === "paused" ? "Resume typing" : "Pause typing"} onClick={() => void togglePause()}><Icon name={state.status === "paused" ? "play" : "pause"} size={14} />{state.status === "paused" ? "Resume" : "Pause"}</button><button className="icon-button" aria-label="Stop typing" title="Stop typing" onClick={() => void stop()}><Icon name="stop" size={14} /></button></> : <button className="icon-button" aria-label="Open main window" title="Open main window" onClick={() => void invoke("show_main")}><Icon name="arrow" size={16} /></button>}
      </div>
    </div>
    <div className="mini-times"><time aria-label="Elapsed typing time" title="Excludes countdowns and pauses">{formatElapsed(timing.elapsedMs)} <span>elapsed</span></time><time aria-label="Estimated time left" title="Estimate based on actual pace, including thinking pauses and revisions">{timing.remainingMs === null ? "Estimating…" : `${timing.remainingMs > 0 ? "~" : ""}${formatElapsed(timing.remainingMs)}`} <span>left</span></time></div>
    {state.errorMessage && <button className="mini-error" title={state.errorMessage} onClick={() => void invoke("show_main")}>Open app for details</button>}
    <div className="mini-progress" role="progressbar" aria-label="Typing progress" aria-valuenow={Math.round(state.percent)} aria-valuemin={0} aria-valuemax={100}><div style={{ width: `${state.percent}%` }} /></div>
  </div>;
}
