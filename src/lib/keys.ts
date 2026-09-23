import { invoke } from "@tauri-apps/api/core";
import type { Direction, KeyAction } from "./types";

const ARROWS: Record<string, Direction> = {
  ArrowUp: "up",
  ArrowDown: "down",
  ArrowLeft: "left",
  ArrowRight: "right",
};

const FONT_UP = new Set(["+", "=", "Add"]);
const FONT_DOWN = new Set(["-", "_", "Subtract"]);

/**
 * Maps a keyboard event to the backend command it should trigger, or null when
 * the key is not bound. Pure so it can be unit tested without a webview.
 */
export function keyToAction(event: KeyboardEvent): KeyAction | null {
  const direction = ARROWS[event.key];
  if (direction && !event.ctrlKey) {
    return { command: "move_corner", args: { direction } };
  }

  if (event.ctrlKey) {
    if (FONT_UP.has(event.key)) {
      return { command: "set_font_size", args: { delta: 1 } };
    }
    if (FONT_DOWN.has(event.key)) {
      return { command: "set_font_size", args: { delta: -1 } };
    }
    if (event.key === "0") {
      return { command: "reset_font_size", args: {} };
    }
    return null;
  }

  if (event.key === "Escape" || event.key === "q" || event.key === "Q") {
    return { command: "quit", args: {} };
  }

  return null;
}

export function handleKeydown(event: KeyboardEvent): void {
  const action = keyToAction(event);
  if (!action) {
    return;
  }
  event.preventDefault();
  void invoke(action.command, action.args);
}
