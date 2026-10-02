import { useState, useEffect, useCallback, useRef } from "react";
import type { Config } from "../lib/types";
import { DEFAULT_CONFIG } from "../lib/types";
import * as commands from "../lib/commands";

export function useConfig() {
  const [config, setConfigState] = useState<Config>(DEFAULT_CONFIG);
  const current = useRef(DEFAULT_CONFIG);
  const [loading, setLoading] = useState(true);
  const [pending, setPending] = useState(0);
  const [error, setError] = useState<string | null>(null);
  const queue = useRef<Promise<void>>(Promise.resolve());
  useEffect(() => {
    let active = true;
    void commands.getConfig().then((cfg) => {
      if (active) { current.current = cfg; setConfigState(cfg); }
    }).catch((err) => { if (active) setError(String(err)); })
      .finally(() => { if (active) setLoading(false); });
    return () => { active = false; };
  }, []);
  const updateConfig = useCallback((updates: Partial<Config>) => {
    const next = { ...current.current, ...updates };
    current.current = next;
    setConfigState(next);
    setPending((n) => n + 1);
    const save = queue.current.catch(() => {}).then(() => commands.setConfig(next));
    queue.current = save;
    void save.then(() => setError(null), (err) => setError(String(err)))
      .finally(() => setPending((n) => n - 1));
  }, []);
  const resetConfig = useCallback(() => updateConfig(DEFAULT_CONFIG), [updateConfig]);
  const flush = useCallback(() => queue.current, []);
  return { config, loading, saving: pending > 0, error, updateConfig, resetConfig, flush };
}
