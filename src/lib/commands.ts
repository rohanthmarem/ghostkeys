import { invoke } from "@tauri-apps/api/core";
import type { BackendTypingState, Config, FileInfo, PlatformInfo } from "./types";

export async function loadFile(path: string): Promise<FileInfo> {
  return invoke<FileInfo>("load_file", { path });
}

export async function startTyping(): Promise<void> {
  return invoke("start_typing");
}

export async function stopTyping(): Promise<void> {
  return invoke("stop_typing");
}

export async function pauseTyping(): Promise<void> {
  return invoke("pause_typing");
}

export async function resumeTyping(): Promise<void> {
  return invoke("resume_typing");
}

export async function getConfig(): Promise<Config> {
  return invoke<Config>("get_config");
}

export async function setConfig(config: Config): Promise<void> {
  return invoke("set_config", { config });
}

export async function getState(): Promise<BackendTypingState> {
  return invoke("get_state");
}

export async function setFileContent(
  content: string,
  fileName: string
): Promise<void> {
  return invoke("set_file_content", { content, fileName });
}

export async function getPlatformInfo(): Promise<PlatformInfo> {
  return invoke("get_platform_info");
}

export async function requestAccessibility(): Promise<boolean> {
  return invoke("request_accessibility");
}

export async function openAccessibilitySettings(): Promise<void> {
  return invoke("open_accessibility_settings");
}
