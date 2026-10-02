import { useEffect, useRef } from "react";
import { Icon } from "./Icon";
export function CountdownOverlay({ countdown, visible, onStop }: { countdown: number; visible: boolean; onStop: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const el = dialog.current!;
    if (visible && !el.open) el.showModal();
    if (!visible && el.open) el.close();
  }, [visible]);
  return <dialog className="countdown-dialog" ref={dialog} onCancel={e => { e.preventDefault(); onStop(); }} aria-label="Starting typing">
    <div className="countdown-content"><span className="countdown-caption">Switch to your text field</span><div className="countdown-number" role="timer" aria-live="polite">{countdown > 0 ? countdown : "…"}</div><button autoFocus className="button secondary" onClick={onStop}><Icon name="close" size={15} />Cancel<kbd>esc</kbd></button></div>
  </dialog>;
}
