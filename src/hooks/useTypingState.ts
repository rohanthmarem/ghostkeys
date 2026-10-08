import { useState, useEffect, useCallback } from "react";
import { listen } from "@tauri-apps/api/event";
import type { TypingState, TypingStatus, TypingProgress } from "../lib/types";
import * as commands from "../lib/commands";

const initialState: TypingState = {
  status: "idle",
  currentChar: 0,
  totalChars: 0,
  percent: 0,
  fileName: null,
  content: null,
  errorMessage: null,
};

export function useTypingState() {
  const [state, setState] = useState<TypingState>(initialState);
  const [countdown, setCountdown] = useState<number>(0);

  // Listen for backend events
  useEffect(() => {
    const unlistenContent = listen<{ content: string; fileName: string }>("content-loaded", (event) => {
      setState((prev) => ({ ...prev, status: "ready", content: event.payload.content, fileName: event.payload.fileName, totalChars: Array.from(event.payload.content).length, currentChar: 0, percent: 0, errorMessage: null }));
    });
    const unlistenProgress = listen<TypingProgress>(
      "typing-progress",
      (event) => {
        setState((prev) => ({
          ...prev,
          currentChar: event.payload.current,
          totalChars: event.payload.total,
          percent: event.payload.percent,
        }));
      }
    );

    const unlistenState = listen<{ status: string }>(
      "typing-state-changed",
      (event) => {
        setState((prev) => ({
          ...prev,
          status: event.payload.status as TypingStatus,
        }));
      }
    );

    const unlistenError = listen<{ message: string }>("typing-error", (event) => {
      setState((prev) => ({
        ...prev,
        status: "error",
        errorMessage: event.payload.message,
      }));
    });

    const unlistenCountdown = listen<{ remaining: number }>(
      "countdown-tick",
      (event) => {
        setCountdown(event.payload.remaining);
      }
    );

    return () => {
      unlistenContent.then((fn) => fn());
      unlistenProgress.then((fn) => fn());
      unlistenState.then((fn) => fn());
      unlistenError.then((fn) => fn());
      unlistenCountdown.then((fn) => fn());
    };
  }, []);

  const loadContent = useCallback(async (content: string, fileName: string | null) => {
    try {
      await commands.setFileContent(content, fileName || "Pasted Text");
      setState((prev) => ({
        ...prev,
        status: "ready",
        fileName: fileName || "Pasted Text",
        content,
        totalChars: Array.from(content).length,
        currentChar: 0,
        percent: 0,
        errorMessage: null,
      }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        status: "error",
        errorMessage: String(error),
      }));
    }
  }, []);

  const updateContent = useCallback(async (content: string) => {
    try {
      const currentFileName = state.fileName || "Pasted Text";
      await commands.setFileContent(content, currentFileName);
      setState((prev) => ({
        ...prev,
        content,
        totalChars: Array.from(content).length,
        currentChar: 0,
        percent: 0,
      }));
    } catch (error) {
      setState((prev) => ({
        ...prev,
        status: "error",
        errorMessage: String(error),
      }));
    }
  }, [state.fileName]);

  const start = useCallback(async () => {
    try {
      await commands.startTyping();
    } catch (error) {
      setState((prev) => ({
        ...prev,
        status: "error",
        errorMessage: String(error),
      }));
    }
  }, []);

  const stop = useCallback(async () => {
    try {
      await commands.stopTyping();
      setState((prev) => ({
        ...prev,
        status: prev.fileName ? "ready" : "idle",
        currentChar: 0,
        percent: 0,
      }));
    } catch (error) {
      console.error("Failed to stop:", error);
    }
  }, []);

  const pause = useCallback(async () => {
    try {
      await commands.pauseTyping();
    } catch (error) {
      console.error("Failed to pause:", error);
    }
  }, []);

  const resume = useCallback(async () => {
    try {
      await commands.resumeTyping();
    } catch (error) {
      console.error("Failed to resume:", error);
    }
  }, []);

  const reset = useCallback(() => {
    setState(initialState);
  }, []);

  return {
    state,
    countdown,
    loadContent,
    updateContent,
    start,
    stop,
    pause,
    resume,
    reset,
  };
}
