import { useState, useCallback, useEffect } from "react";
import { useDropzone } from "react-dropzone";
import { isTauri } from "@tauri-apps/api/core";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import * as commands from "../lib/commands";

interface ContentInputProps {
  onContentLoad: (content: string, fileName: string | null) => void;
  fileName: string | null;
  disabled?: boolean;
}

type InputMode = "file" | "paste";

export function ContentInput({
  onContentLoad,
  fileName,
  disabled,
}: ContentInputProps) {
  const [mode, setMode] = useState<InputMode>("file");
  const [pasteText, setPasteText] = useState("");
  const [fileError, setFileError] = useState<string | null>(null);
  const [isNativeDragActive, setNativeDragActive] = useState(false);

  const onDrop = useCallback(
    (acceptedFiles: File[]) => {
      const file = acceptedFiles[0];
      if (!file) return;

      setFileError(null);
      const reader = new FileReader();
      reader.onload = () => {
        const content = reader.result as string;
        onContentLoad(content, file.name);
      };
      reader.onerror = () => {
        setFileError(`Failed to read file: ${reader.error?.message || 'Unknown error'}`);
      };
      reader.readAsText(file);
    },
    [onContentLoad]
  );

  const { getRootProps, getInputProps, rootRef, isDragActive: isBrowserDragActive } = useDropzone({
    onDrop,
    accept: {
      "text/markdown": [".md"],
      "text/plain": [".txt"],
    },
    multiple: false,
    disabled,
    onDropRejected: () => setFileError("Select a single .txt or .md file."),
  });

  // Tauri receives native Finder drops before the WebView's HTML drop handler.
  useEffect(() => {
    if (!isTauri() || disabled || mode !== "file") return;
    let active = true;
    const unlisten = getCurrentWebview().onDragDropEvent(async ({ payload }) => {
      if (!active) return;
      if (payload.type === "leave") {
        setNativeDragActive(false);
        return;
      }

      const bounds = rootRef.current?.getBoundingClientRect();
      const x = payload.position.x / window.devicePixelRatio;
      const y = payload.position.y / window.devicePixelRatio;
      const inside = !!bounds && x >= bounds.left && x <= bounds.right &&
        y >= bounds.top && y <= bounds.bottom;
      setNativeDragActive(inside && payload.type !== "drop");
      if (payload.type !== "drop" || !inside) return;

      if (payload.paths.length !== 1 || !/\.(txt|md)$/i.test(payload.paths[0])) {
        setFileError("Select a single .txt or .md file.");
        return;
      }

      try {
        setFileError(null);
        const file = await commands.loadFile(payload.paths[0]);
        if (active) onContentLoad(file.content, file.name);
      } catch (error) {
        if (active) setFileError(String(error));
      }
    }).catch((error) => {
      if (active) setFileError(`File drop is unavailable: ${String(error)}`);
      return () => {};
    });

    return () => {
      active = false;
      setNativeDragActive(false);
      void unlisten.then((stop) => stop());
    };
  }, [disabled, mode, onContentLoad, rootRef]);

  const isDragActive = isBrowserDragActive || isNativeDragActive;

  const handlePasteSubmit = useCallback(() => {
    if (pasteText.trim()) {
      onContentLoad(pasteText, null);
    }
  }, [pasteText, onContentLoad]);

  return (
    <div className="space-y-3">
      {/* Tab buttons */}
      <div className="flex border-b border-ghost-700">
        <button
          onClick={() => setMode("file")}
          disabled={disabled}
          className={`
            flex items-center gap-2 px-4 py-2 text-sm font-medium
            border-b-2 transition-colors -mb-px
            ${
              mode === "file"
                ? "border-accent-primary text-ghost-100"
                : "border-transparent text-ghost-500 hover:text-ghost-300"
            }
            ${disabled ? "opacity-50 cursor-not-allowed" : ""}
          `}
        >
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
            />
          </svg>
          File
        </button>
        <button
          onClick={() => setMode("paste")}
          disabled={disabled}
          className={`
            flex items-center gap-2 px-4 py-2 text-sm font-medium
            border-b-2 transition-colors -mb-px
            ${
              mode === "paste"
                ? "border-accent-primary text-ghost-100"
                : "border-transparent text-ghost-500 hover:text-ghost-300"
            }
            ${disabled ? "opacity-50 cursor-not-allowed" : ""}
          `}
        >
          <svg
            className="w-4 h-4"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
            />
          </svg>
          Paste
        </button>
      </div>

      {/* Content area */}
      {mode === "file" ? (
        <div
          {...getRootProps()}
          className={`
            border border-dashed rounded-lg p-8 text-center cursor-pointer
            transition-all duration-200 ease-in-out
            ${
              isDragActive
                ? "border-accent-primary bg-accent-primary/5"
                : "border-ghost-700 hover:border-ghost-500 hover:bg-ghost-900/50"
            }
            ${disabled ? "opacity-50 cursor-not-allowed" : ""}
          `}
        >
          <input {...getInputProps()} />
          <div className="flex flex-col items-center gap-3">
            <svg
              className={`w-10 h-10 ${
                isDragActive ? "text-accent-primary" : "text-ghost-500"
              }`}
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z"
              />
            </svg>
            {fileName ? (
              <div>
                <p className="text-ghost-200 font-medium">{fileName}</p>
                <p className="text-ghost-500 text-sm mt-1">
                  Drop a new file to replace
                </p>
              </div>
            ) : (
              <div>
                <p className="text-ghost-300">
                  {isDragActive
                    ? "Drop the file here..."
                    : "Drag & drop a text file here"}
                </p>
                <p className="text-ghost-500 text-sm mt-1">
                  or click to select (.txt, .md)
                </p>
              </div>
            )}
            {fileError && (
              <p className="text-accent-error text-sm mt-2">{fileError}</p>
            )}
          </div>
        </div>
      ) : (
        <div className="space-y-3">
          <textarea
            value={pasteText}
            onChange={(e) => setPasteText(e.target.value)}
            disabled={disabled}
            placeholder="Paste or type your text here..."
            className={`
              w-full h-32 p-4 rounded-lg resize-none
              bg-ghost-900 border border-ghost-700
              text-ghost-200 placeholder-ghost-600
              font-serif text-sm leading-relaxed
              focus:outline-none focus:border-accent-primary/50
              ${disabled ? "opacity-50 cursor-not-allowed" : ""}
            `}
          />
          <div className="flex items-center justify-between">
            <p className="text-xs text-ghost-500">
              {pasteText.length > 0 ? (
                <span className="font-mono">
                  {Array.from(pasteText).length.toLocaleString()} characters
                </span>
              ) : (
                "Paste from clipboard or type directly"
              )}
            </p>
            <button
              onClick={handlePasteSubmit}
              disabled={disabled || !pasteText.trim()}
              className={`
                px-4 py-2 text-sm rounded-lg font-medium
                transition-all flex items-center gap-2
                ${
                  pasteText.trim() && !disabled
                    ? "bg-accent-primary hover:bg-accent-primary/80 text-white"
                    : "bg-ghost-800 text-ghost-500 cursor-not-allowed"
                }
              `}
            >
              <svg
                className="w-4 h-4"
                fill="none"
                stroke="currentColor"
                viewBox="0 0 24 24"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M5 13l4 4L19 7"
                />
              </svg>
              Load Text
            </button>
          </div>
        </div>
      )}
    </div>
  );
}
