import type { CSSProperties } from "react";
const paths = {
  open: <><path d="M3 7h6l2 2h10l-3 10H3V7Z" /><path d="M3 7V5h7l2 2h7v2" /></>,
  settings: <><path d="M4 7h16M4 17h16" /><circle cx="9" cy="7" r="3" /><circle cx="15" cy="17" r="3" /></>,
  mini: <><rect x="3" y="4" width="18" height="16" rx="3" /><path d="M13 12h8v8h-8z" /></>,
  close: <path d="m6 6 12 12M6 18 18 6" />,
  play: <path d="m8 5 11 7-11 7V5Z" />,
  pause: <><path d="M8 5v14M16 5v14" /></>,
  stop: <rect x="6" y="6" width="12" height="12" rx="1" />,
  arrow: <path d="M5 12h14m-6-6 6 6-6 6" />,
  check: <path d="m5 12 4 4L19 6" />,
  clock: <><circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" /></>,
  trash: <><path d="M4 7h16M9 7V4h6v3M6 7l1 13h10l1-13M10 10v7M14 10v7" /></>,
  chevron: <path d="m8 10 4 4 4-4" />,
};
export function Icon({ name, size = 18, style }: { name: keyof typeof paths; size?: number; style?: CSSProperties }) {
  return <svg width={size} height={size} viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.65" strokeLinecap="round" strokeLinejoin="round" aria-hidden="true" style={style}>{paths[name]}</svg>;
}
export function Mark() {
  return <svg className="wordmark-icon" width="26" height="26" viewBox="0 0 26 26" fill="none" aria-hidden="true"><rect width="26" height="26" rx="7" fill="currentColor" /><path d="M7.5 16V10m5.5 8V8m5.5 8v-6" stroke="var(--paper)" strokeWidth="2" strokeLinecap="round" /></svg>;
}
export const shortcutLabel = (value: string) => value.replace(/Control/g, "⌃").replace(/Alt/g, "⌥").replace(/Super|Meta/g, "⌘").replace(/Shift/g, "⇧").replace(/\+/g, "");
