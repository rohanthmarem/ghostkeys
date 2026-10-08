import { useState } from "react";
import { ContentInput } from "./components/ContentInput";
import { TextEditor } from "./components/TextEditor";
import { TypingControls } from "./components/TypingControls";
import { SettingsPanel } from "./components/SettingsPanel";
import { StatusBar } from "./components/StatusBar";
import { CountdownOverlay } from "./components/CountdownOverlay";
import { MacPermissions } from "./components/MacPermissions";
import { MacBackgroundTarget } from "./components/MacBackgroundTarget";
import { useTypingState } from "./hooks/useTypingState";
import { useConfig } from "./hooks/useConfig";
import "./styles/globals.css";

function App() {
  const [capturing, setCapturing] = useState(false);
  const { state, countdown, loadContent, updateContent, start, stop, pause, resume } =
    useTypingState();
  const { config, updateConfig, resetConfig } = useConfig();

  const isTypingOrCountdown =
    capturing || state.status === "typing" || state.status === "countdown" || state.status === "paused";

  return (
    <div className="min-h-screen bg-ghost-950 text-ghost-100 p-6">
      {/* Header */}
      <header className="flex items-center gap-3 mb-8" data-tauri-drag-region>
        <div className="text-3xl opacity-80">&#128123;</div>
        <div>
          <h1 className="text-2xl font-semibold text-ghost-50 tracking-tight">ghostkeys</h1>
          <p className="text-ghost-400 text-sm">Human typing simulator</p>
        </div>
      </header>

      <div className="space-y-6">
        <MacPermissions />
        <MacBackgroundTarget disabled={isTypingOrCountdown} onCapturingChange={setCapturing} />
        {/* Content Input (File Drop or Paste) */}
        <section>
          <ContentInput
            onContentLoad={loadContent}
            fileName={state.fileName}
            disabled={isTypingOrCountdown}
          />
        </section>

        {/* Text Editor/Preview */}
        {state.content && (
          <section>
            <TextEditor
              content={state.content}
              fileName={state.fileName}
              onContentChange={updateContent}
              disabled={isTypingOrCountdown}
            />
          </section>
        )}

        {/* Status */}
        <section>
          <StatusBar
            status={state.status}
            currentChar={state.currentChar}
            totalChars={state.totalChars}
            percent={state.percent}
            errorMessage={state.errorMessage}
          />
        </section>

        {/* Controls */}
        <section>
          <TypingControls
            status={state.status}
            onStart={start}
            onStop={stop}
            onPause={pause}
            onResume={resume}
            hasFile={!capturing && !!state.content}
          />
        </section>

        {/* Hotkey hint */}
        <div className="text-center text-ghost-500 text-sm">
          Press <kbd className="px-2 py-1 bg-ghost-800 rounded text-ghost-300 font-mono text-xs">{navigator.userAgent.includes("Mac") ? "Control+Option+S" : "Ctrl+Alt+S"}</kbd> to start/stop
        </div>

        {/* Settings */}
        <section className="border-t border-ghost-800 pt-6">
          <SettingsPanel
            config={config}
            onUpdate={updateConfig}
            onReset={resetConfig}
            disabled={isTypingOrCountdown}
          />
        </section>
      </div>

      {/* Countdown Overlay */}
      <CountdownOverlay
        countdown={countdown}
        visible={state.status === "countdown"}
      />
    </div>
  );
}

export default App;
