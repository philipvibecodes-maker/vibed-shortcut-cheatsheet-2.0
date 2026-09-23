import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { CheatsheetState } from "./types";

export const UPDATE_EVENT = "cheatsheet://update";

export function getState(): Promise<CheatsheetState> {
  return invoke<CheatsheetState>("get_state");
}

export function onUpdate(
  handler: (state: CheatsheetState) => void,
): Promise<UnlistenFn> {
  return listen<CheatsheetState>(UPDATE_EVENT, (event) => handler(event.payload));
}

export function reportContentSize(width: number, height: number): Promise<void> {
  return invoke("report_content_size", { width, height });
}
