import { useEffect, useRef, useState } from "react";
import { isTauri } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { Icon } from "./Icon";
import { loadFile } from "../lib/commands";

interface Props {
  content: string;
  fileName: string | null;
  disabled: boolean;
  selectedCount: number;
  onSelection: (start: number, end: number) => void;
  onChange: (content: string, fileName?: string) => void;
}

export function ComposeEditor({ content, fileName, disabled, onChange, selectedCount, onSelection }: Props) {
  const [dragging, setDragging] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [opening, setOpening] = useState(false);
  const fallback = useRef<HTMLInputElement>(null);
  const editor = useRef<HTMLTextAreaElement>(null);
  const chars = Array.from(content).length;
  const words = content.trim() ? content.trim().split(/\s+/u).length : 0;

  const readPath = async (path: string) => {
    if (!/\.(txt|md)$/i.test(path)) throw new Error("Choose a .txt or .md file.");
    const file = await loadFile(path);
    onChange(file.content, file.name);
  };

  useEffect(() => {
    if (!isTauri() || disabled) return;
    let active = true;
    const subscription = getCurrentWebview().onDragDropEvent(async ({ payload }) => {
      if (!active) return;
      setDragging(payload.type === "enter" || payload.type === "over");
      if (payload.type !== "drop") return;
      try {
        setError(null);
        if (payload.paths.length !== 1) throw new Error("Drop one text file at a time.");
        await readPath(payload.paths[0]);
      } catch (e) { if (active) setError(String(e)); }
    });
    void subscription.catch((e) => { if (active) setError(String(e)); });
    return () => { active = false; void subscription.then(fn => fn()).catch(() => {}); };
  }, [disabled, onChange]);
  const chooseFile = async () => {
    if (!isTauri()) { fallback.current?.click(); return; }
    setOpening(true); setError(null);
    try {
      const path = await open({ multiple: false, filters: [{ name: "Text files", extensions: ["txt", "md"] }] });
      if (typeof path === "string") await readPath(path);
    } catch (e) { setError(String(e)); } finally { setOpening(false); }
  };
  const updateSelection = (el: HTMLTextAreaElement) => {
    if (!disabled) onSelection(Array.from(el.value.slice(0, el.selectionStart)).length, Array.from(el.value.slice(0, el.selectionEnd)).length);
  };
  return <section className={`document ${dragging ? "is-dragging" : ""}`} aria-label="Draft">
    <header className="document-header">
      <h2 title={fileName || "Untitled"}>{fileName && fileName !== "Untitled" ? fileName : "Untitled"}</h2>
      <div className="document-actions">
        <button className="text-button open-file" onClick={() => void chooseFile()} disabled={disabled || opening}><Icon name="open" size={17} />{opening ? "Opening…" : "Open file"}</button>
        {content && <button className="icon-button" aria-label="Clear draft" title="Clear draft" disabled={disabled} onClick={() => { onChange("", "Untitled"); editor.current?.focus(); }}><Icon name="trash" size={16} /></button>}
      </div>
    </header>
    <input ref={fallback} type="file" accept=".txt,.md" hidden onChange={async e => {
      const file = e.target.files?.[0]; if (!file) return;
      try { onChange(await file.text(), file.name); setError(null); } catch (err) { setError(String(err)); }
      e.target.value = "";
    }} />
    <div className="document-body">
      <textarea ref={editor} aria-label="Text to type" value={content} readOnly={disabled} spellCheck={false}
        onSelect={e => updateSelection(e.currentTarget)} onKeyUp={e => updateSelection(e.currentTarget)} onMouseUp={e => updateSelection(e.currentTarget)}
        placeholder="Paste or write your text…"
        onChange={e => { setError(null); onChange(e.target.value); }} />
      {dragging && <div className="drop-overlay"><Icon name="open" size={28} />Drop your text file</div>}
    </div>
    <footer className="document-footer">
      <span>{words.toLocaleString()} {words === 1 ? "word" : "words"}<span className="meta-divider">/</span>{chars.toLocaleString()} characters</span>
      {selectedCount > 0 && <button className="selection-chip" disabled={disabled} title="Use the entire draft" onClick={() => { editor.current?.setSelectionRange(0, 0); onSelection(0, 0); }}>{selectedCount.toLocaleString()} selected<Icon name="close" size={13} /></button>}
    </footer>
    {error && <p className="notice error" role="alert">{error}</p>}
  </section>;
}
