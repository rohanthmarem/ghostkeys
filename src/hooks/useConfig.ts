import { useState, useEffect, useCallback } from "react";
import type { Config } from "../lib/types";
import { DEFAULT_CONFIG } from "../lib/types";
import * as commands from "../lib/commands";
import { listen } from "@tauri-apps/api/event";

export function useConfig() {
  const [config, setConfigState] = useState<Config>(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(true);

  // Load config on mount
  useEffect(() => {
    const unlisten = listen<Config>("config-changed", (event) => setConfigState(event.payload));
    commands
      .getConfig()
      .then((cfg) => {
        setConfigState(cfg);
        setLoading(false);
      })
      .catch((err) => {
        console.error("Failed to load config:", err);
        setLoading(false);
      });
    return () => { unlisten.then((fn) => fn()); };
  }, []);

  const updateConfig = useCallback(async (updates: Partial<Config>) => {
    const newConfig = { ...config, ...updates };
    setConfigState(newConfig);
    try {
      await commands.setConfig(newConfig);
    } catch (error) {
      console.error("Failed to save config:", error);
    }
  }, [config]);

  const resetConfig = useCallback(async () => {
    setConfigState(DEFAULT_CONFIG);
    try {
      await commands.setConfig(DEFAULT_CONFIG);
    } catch (error) {
      console.error("Failed to reset config:", error);
    }
  }, []);

  return {
    config,
    loading,
    updateConfig,
    resetConfig,
  };
}
