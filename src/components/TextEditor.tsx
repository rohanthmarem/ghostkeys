import { useState, useCallback, useEffect, useMemo } from "react";

interface TextEditorProps {
  content: string | null;
  fileName: string | null;
  onContentChange: (content: string) => void;
  disabled?: boolean;
}

export function TextEditor({
  content,
  fileName,
  onContentChange,
  disabled,
}: TextEditorProps) {
  const [localContent, setLocalContent] = useState(content || "");
  const [isEditing, setIsEditing] = useState(false);

  // Sync local content when external content changes (file load)
  useEffect(() => {
    setLocalContent(content || "");
  }, [content]);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLTextAreaElement>) => {
      const newContent = e.target.value;
      setLocalContent(newContent);
      setIsEditing(true);
    },
    []
  );

  const handleApply = useCallback(() => {
    onContentChange(localContent);
    setIsEditing(false);
  }, [localContent, onContentChange]);

  const handleReset = useCallback(() => {
    setLocalContent(content || "");
    setIsEditing(false);
  }, [content]);

  // Memoize text statistics calculation
  const { charCount, wordCount, estimatedMinutes } = useMemo(() => {
    const chars = Array.from(localContent).length;
    const words = localContent
      .trim()
      .split(/\s+/)
      .filter((w) => w.length > 0).length;
    return {
      charCount: chars,
      wordCount: words,
      estimatedMinutes: Math.ceil(words / 60),
    };
  }, [localContent]);

  if (!content && !localContent) {
    return (
      <div className="rounded-lg border border-ghost-700 bg-ghost-900/50 p-6 text-center">
        <p className="text-ghost-500">
          No content loaded. Drop a file or paste text above.
        </p>
      </div>
    );
  }

  return (
    <div className="space-y-3">
      {/* Header with stats */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <h3 className="text-sm font-medium text-ghost-300">Preview & Edit</h3>
          {fileName && (
            <span className="text-xs text-ghost-500 bg-ghost-800 px-2 py-1 rounded">
              {fileName}
            </span>
          )}
        </div>
        <div className="flex items-center gap-4 text-xs text-ghost-500">
          <span className="font-mono">{charCount.toLocaleString()} chars</span>
          <span className="font-mono">{wordCount.toLocaleString()} words</span>
          <span className="font-mono">~{estimatedMinutes} min</span>
        </div>
      </div>

      {/* Text area */}
      <textarea
        value={localContent}
        onChange={handleChange}
        disabled={disabled}
        placeholder="Your text will appear here..."
        className={`
          w-full h-48 p-4 rounded-lg resize-none
          bg-ghost-900 border border-ghost-700
          text-ghost-200 placeholder-ghost-600
          font-serif text-sm leading-relaxed
          focus:outline-none focus:border-accent-primary/50
          ${disabled ? "opacity-50 cursor-not-allowed" : ""}
        `}
      />

      {/* Action buttons (only show when editing) */}
      {isEditing && !disabled && (
        <div className="flex items-center justify-end gap-2">
          <button
            onClick={handleReset}
            className="px-3 py-1.5 text-sm rounded
                       bg-ghost-800 hover:bg-ghost-700
                       text-ghost-300 transition-colors"
          >
            Reset
          </button>
          <button
            onClick={handleApply}
            className="px-3 py-1.5 text-sm rounded
                       bg-accent-primary hover:bg-accent-primary/80
                       text-white transition-colors"
          >
            Apply Changes
          </button>
        </div>
      )}
    </div>
  );
}
