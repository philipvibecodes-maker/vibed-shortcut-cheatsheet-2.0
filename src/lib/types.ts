export interface Shortcut {
  action: string;
  keys: string;
}

export interface CheatsheetState {
  displayName: string;
  shortcuts: Shortcut[];
  fontSize: number;
}

export type Direction = "up" | "down" | "left" | "right";

export type KeyAction =
  | { command: "move_corner"; args: { direction: Direction } }
  | { command: "set_font_size"; args: { delta: number } }
  | { command: "reset_font_size"; args: Record<string, never> }
  | { command: "quit"; args: Record<string, never> };
