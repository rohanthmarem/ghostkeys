import { useState, useEffect, useCallback, useRef } from "react";
import { invoke, isTauri } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
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
  const [loading, setLoading] = useState(true);
  const [pendingSaves, setPendingSaves] = useState(0);
  const saveQueue = useRef<Promise<void>>(Promise.resolve());
  const draftName = useRef("Untitled");
  const [selectedCount, setSelectedCount] = useState(0);
  const selection = useRef("0:0");
  const [countdown, setCountdown] = useState<number>(0);

  // Listen for backend events
  useEffect(() => {
    if (!isTauri()) return;
    let active = true;
    let revision = 0;
    const unlisteners: UnlistenFn[] = [];
    const register = (listener: Promise<UnlistenFn>) => listener.then((unlisten) => {
      if (active) unlisteners.push(unlisten);
      else unlisten();
    });

    const unlistenProgress = listen<TypingProgress>(
      "typing-progress",
      (event) => {
        if (!active) return;
        revision += 1;
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
        if (!active) return;
        revision += 1;
        setState((prev) => ({
          ...prev,
          status: event.payload.status as TypingStatus,
          errorMessage: event.payload.status === "error" ? prev.errorMessage : null,
        }));
      }
    );

    const unlistenError = listen<{ message: string }>("typing-error", (event) => {
      if (!active) return;
      revision += 1;
      setState((prev) => ({
        ...prev,
        status: "error",
        errorMessage: event.payload.message,
      }));
    });

    const unlistenCountdown = listen<{ remaining: number }>(
      "countdown-tick",
      (event) => {
        if (!active) return;
        setCountdown(event.payload.remaining);
      }
    );

    const unlistenNotice = listen<string>("session-notice", event => { if (active) setState(prev => ({ ...prev, errorMessage: event.payload })); });

    // Subscribe first, then restore state without overwriting newer events.
    void Promise.all([
      register(unlistenProgress),
      register(unlistenState),
      register(unlistenError),
      register(unlistenCountdown),
      register(unlistenNotice),
    ]).then(async () => {
      const snapshotRevision = revision;
      const snapshot = await commands.getState();
      if (!active || snapshotRevision !== revision) return;
      draftName.current = snapshot.file_name || "Untitled";
      setState({
        status: snapshot.status,
        currentChar: snapshot.current_char,
        totalChars: snapshot.total_chars,
        percent: snapshot.total_chars > 0 ? snapshot.current_char / snapshot.total_chars * 100 : 0,
        fileName: snapshot.file_name,
        content: snapshot.content,
        errorMessage: snapshot.error_message,
      });
    }).catch((error) => setState((prev) => ({ ...prev, errorMessage: String(error) })))
      .finally(() => { if (active) setLoading(false); });

    return () => {
      active = false;
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  const changeContent = useCallback((content: string, fileName?: string) => {
    selection.current = "0:0"; setSelectedCount(0);
    content = content.replace(/\r\n?/g, "\n");
    if (fileName !== undefined) draftName.current = fileName;
    const name = draftName.current;
    setPendingSaves((count) => count + 1);
    setState((prev) => ({ ...prev, content, fileName: name,
      status: content ? "ready" : "idle", currentChar: 0, totalChars: Array.from(content).length,
      percent: 0, errorMessage: null }));
    const save = saveQueue.current.catch(() => {}).then(() => commands.setFileContent(content, name));
    saveQueue.current = save;
    void save.catch(async (error) => {
      const snapshot = await commands.getState().catch(() => null);
      setState((prev) => ({ ...prev, status: snapshot?.status ?? prev.status, errorMessage: String(error) }));
    }).finally(() => setPendingSaves((count) => count - 1));
  }, []);

  const changeSelection = useCallback((start: number, end: number) => {
    const key = start === end ? "0:0" : `${start}:${end}`;
    if (selection.current === key) return;
    selection.current = key;
    setSelectedCount(end - start);
    setPendingSaves(n => n + 1);
    const save = saveQueue.current.catch(() => {}).then(() => invoke<void>("set_selection", { start, end }));
    saveQueue.current = save;
    void save.catch(e => setState(prev => ({ ...prev, errorMessage: String(e) }))).finally(() => setPendingSaves(n => n - 1));
  }, []);

  const start = useCallback(async () => {
    try {
      await saveQueue.current;
      await commands.startTyping();
    } catch (error) {
      const snapshot = await commands.getState().catch(() => null);
      setState((prev) => ({
        ...prev,
        status: snapshot && ["typing", "countdown", "paused"].includes(snapshot.status) ? snapshot.status : "error",
        errorMessage: String(error),
      }));
    }
  }, []);

  const stop = useCallback(async () => {
    try {
      await commands.stopTyping();
    } catch (error) {
      setState((prev) => ({ ...prev, errorMessage: `Could not stop: ${String(error)}` }));
    }
  }, []);

  const pause = useCallback(async () => {
    try {
      await commands.pauseTyping();
    } catch (error) {
      setState((prev) => ({ ...prev, errorMessage: `Could not pause: ${String(error)}` }));
    }
  }, []);

  const resume = useCallback(async () => {
    try {
      await commands.resumeTyping();
    } catch (error) {
      setState((prev) => ({
        ...prev,
        errorMessage: String(error),
      }));
    }
  }, []);

  const reset = useCallback(() => {
    setState(initialState);
  }, []);

  return {
    state,
    countdown,
    changeContent,
    changeSelection,
    selectedCount,
    loading,
    isSaving: pendingSaves > 0,
    start,
    stop,
    pause,
    resume,
    reset,
  };
}
