import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { useTypingState } from "./hooks/useTypingState";
import { Icon, Mark } from "./components/Icon";
import "./styles/globals.css";
export default function Widget() {
  const { state, countdown, pause, resume, stop } = useTypingState();
  const active = ["typing", "countdown", "paused"].includes(state.status);
  const label = { idle: "Ready", ready: "Ready", countdown: `Starting in ${countdown || "…"}`, typing: "Typing", paused: "Paused", done: "Finished", error: "Needs attention" }[state.status];
  return <div className="mini-window">
    <header data-tauri-drag-region><span data-tauri-drag-region>ghostkeys</span><button className="icon-button" aria-label="Hide mini window" title="Hide mini window" onClick={() => void getCurrentWindow().hide()}><Icon name="close" size={13} /></button></header>
    <div className="mini-content"><Mark /><div className="mini-status"><strong>{label}</strong><span>{state.errorMessage ? "Open app for details" : `${state.currentChar} / ${state.totalChars} characters`}</span></div><div className="mini-actions">
      {active ? <>{state.status !== "countdown" && <button className="icon-button" aria-label={state.status === "paused" ? "Resume" : "Pause"} title={state.status === "paused" ? "Resume" : "Pause"} onClick={() => void (state.status === "paused" ? resume() : pause())}><Icon name={state.status === "paused" ? "play" : "pause"} size={14} /></button>}<button className="icon-button" aria-label="Stop typing" title="Stop typing" onClick={() => void stop()}><Icon name="stop" size={14} /></button></> : <button className="icon-button" aria-label="Open main window" title="Open main window" onClick={() => void invoke("show_main")}><Icon name="arrow" size={16} /></button>}
    </div></div><div className="mini-progress" role="progressbar" aria-label="Typing progress" aria-valuenow={Math.round(state.percent)} aria-valuemin={0} aria-valuemax={100}><div style={{ width: `${state.percent}%` }} /></div>
  </div>;
}
