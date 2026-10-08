import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

export function MacPermissions() {
  const [needsPermission, setNeedsPermission] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let active = true;
    const check = async () => {
      try {
        const status = await invoke<{ isMac: boolean; accessibilityGranted: boolean }>("get_platform_status");
        if (active) setNeedsPermission(status.isMac && !status.accessibilityGranted);
      } catch (error) {
        console.error("Could not check keyboard permission:", error);
      }
    };
    void check();
    const interval = window.setInterval(check, 2000);
    return () => { active = false; window.clearInterval(interval); };
  }, []);

  if (!needsPermission) return null;

  return (
    <aside className="rounded-lg border border-accent-warning/40 bg-ghost-900 p-4 space-y-3" role="status">
      <h2 className="text-ghost-100 font-semibold">Allow typing on your Mac</h2>
      <p className="text-sm text-ghost-300 leading-relaxed">
        Enable ghostkeys in System Settings → Privacy &amp; Security → Accessibility.
        On newer macOS, this is called Device Control and Data Access.
        If ghostkeys is missing, use the + button to add it from Applications.
      </p>
      <button className="rounded-lg bg-accent-primary px-4 py-2 text-sm text-white" onClick={async () => {
        try { await invoke("open_accessibility_settings"); setError(null); }
        catch (error) { setError(String(error)); }
      }}>Open System Settings</button>
      {error && <p className="text-sm text-accent-error">{error}</p>}
    </aside>
  );
}
