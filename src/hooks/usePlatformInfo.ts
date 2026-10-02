import { useCallback, useEffect, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { PlatformInfo } from "../lib/types";
import * as commands from "../lib/commands";

export function usePlatformInfo() {
  const [platformInfo, setPlatformInfo] = useState<PlatformInfo | null>(null);
  const [permissionError, setPermissionError] = useState<string | null>(null);
  const [openingSettings, setOpeningSettings] = useState(false);
  const needsAccessibility =
    platformInfo?.platform === "macos" && !platformInfo.accessibilityGranted;

  useEffect(() => {
    if (!isTauri()) return;
    let active = true;

    const refresh = async () => {
      try {
        const info = await commands.getPlatformInfo();
        if (active) setPlatformInfo(info);
      } catch (error) {
        console.error("Failed to check platform permissions:", error);
      }
    };

    void refresh();
    window.addEventListener("focus", refresh);
    const unlistenFocus = getCurrentWindow().onFocusChanged(({ payload: focused }) => {
      if (focused && active) void refresh();
    }).catch((error) => {
      console.error("Failed to listen for window focus:", error);
      return () => {};
    });
    // System Settings can grant access while this window remains visible.
    const interval = needsAccessibility ? window.setInterval(refresh, 2000) : null;

    return () => {
      active = false;
      window.removeEventListener("focus", refresh);
      void unlistenFocus.then((unlisten) => unlisten());
      if (interval !== null) window.clearInterval(interval);
    };
  }, [needsAccessibility]);

  const openSettings = useCallback(async () => {
    setOpeningSettings(true);
    setPermissionError(null);
    try {
      const granted = await commands.requestAccessibility();
      if (!granted) await commands.openAccessibilitySettings();
      setPlatformInfo(await commands.getPlatformInfo());
    } catch (error) {
      setPermissionError(String(error));
    } finally {
      setOpeningSettings(false);
    }
  }, []);

  return { platformInfo, needsAccessibility, permissionError, openingSettings, openSettings };
}
