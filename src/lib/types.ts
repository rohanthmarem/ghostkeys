// Typing state
export type TypingStatus =
  | "idle"
  | "ready"
  | "countdown"
  | "typing"
  | "paused"
  | "done"
  | "error";

export interface TypingState {
  status: TypingStatus;
  currentChar: number;
  totalChars: number;
  percent: number;
  fileName: string | null;
  content: string | null;
  errorMessage: string | null;
}

export interface TypingProgress {
  current: number;
  total: number;
  percent: number;
}

// Configuration
export interface Config {
  baseWpm: number;
  wpmVariance: number;
  mistakeRate: number;
  correctionRate: number;
  punctuationPause: number;
  paragraphPause: number;
  thinkingPauseChance: number;
  thinkingPauseDuration: number;
  burstTyping: boolean;
  countdownSeconds: number;
}

export const DEFAULT_CONFIG: Config = {
  baseWpm: 60,
  wpmVariance: 0.3,
  mistakeRate: 0.03,
  correctionRate: 0.7,
  punctuationPause: 300,
  paragraphPause: 800,
  thinkingPauseChance: 0.02,
  thinkingPauseDuration: 1500,
  burstTyping: true,
  countdownSeconds: 3,
};

// File info returned from backend
export interface FileInfo {
  name: string;
  content: string;
  charCount: number;
}

export interface BackendTypingState {
  status: TypingStatus;
  current_char: number;
  total_chars: number;
  file_name: string | null;
  content: string | null;
  error_message: string | null;
}

export interface PlatformInfo {
  platform: "macos" | "windows" | "linux";
  accessibilityGranted: boolean;
  shortcutLabel: string;
}

// Widget position
export interface WidgetPosition {
  x: number;
  y: number;
}
