import type { TypingStatus } from "../lib/types";

interface TypingControlsProps {
  status: TypingStatus;
  onStart: () => void;
  onStop: () => void;
  onPause: () => void;
  onResume: () => void;
  hasFile: boolean;
}

export function TypingControls({
  status,
  onStart,
  onStop,
  onPause,
  onResume,
  hasFile,
}: TypingControlsProps) {
  const isTyping = status === "typing" || status === "countdown";
  const isPaused = status === "paused";

  return (
    <div className="flex gap-3">
      {/* Start/Stop Button */}
      {isTyping || isPaused ? (
        <button
          onClick={onStop}
          className="flex-1 py-3 px-6 rounded-lg font-medium transition-all
                     bg-accent-error hover:bg-accent-error/80 text-white
                     flex items-center justify-center gap-2"
        >
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
            <rect x="6" y="6" width="12" height="12" rx="1" />
          </svg>
          Stop
        </button>
      ) : (
        <button
          onClick={onStart}
          disabled={!hasFile}
          className={`flex-1 py-3 px-6 rounded-lg font-medium transition-all
                     flex items-center justify-center gap-2
                     ${
                       hasFile
                         ? "bg-accent-primary hover:bg-accent-primary/80 text-white"
                         : "bg-ghost-800 text-ghost-500 cursor-not-allowed"
                     }`}
        >
          <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
            <path d="M8 5v14l11-7z" />
          </svg>
          Start Typing
        </button>
      )}

      {/* Pause/Resume Button */}
      {(isTyping || isPaused) && (
        <button
          onClick={isPaused ? onResume : onPause}
          disabled={status === "countdown"}
          className={`py-3 px-6 rounded-lg font-medium transition-all
                     flex items-center justify-center gap-2
                     ${
                       status === "countdown"
                         ? "bg-ghost-800 text-ghost-500 cursor-not-allowed"
                         : "bg-ghost-700 hover:bg-ghost-600 text-ghost-100"
                     }`}
        >
          {isPaused ? (
            <>
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <path d="M8 5v14l11-7z" />
              </svg>
              Resume
            </>
          ) : (
            <>
              <svg className="w-5 h-5" fill="currentColor" viewBox="0 0 24 24">
                <rect x="6" y="5" width="4" height="14" rx="1" />
                <rect x="14" y="5" width="4" height="14" rx="1" />
              </svg>
              Pause
            </>
          )}
        </button>
      )}
    </div>
  );
}
