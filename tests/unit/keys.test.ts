import { describe, expect, it, vi } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));

const { keyToAction } = await import("../../src/lib/keys");

function key(init: KeyboardEventInit & { key: string }): KeyboardEvent {
  return new KeyboardEvent("keydown", init);
}

describe("keyToAction", () => {
  it("maps arrow keys to corner moves", () => {
    expect(keyToAction(key({ key: "ArrowLeft" }))).toEqual({
      command: "move_corner",
      args: { direction: "left" },
    });
    expect(keyToAction(key({ key: "ArrowDown" }))).toEqual({
      command: "move_corner",
      args: { direction: "down" },
    });
  });

  it("maps ctrl +/- to font size changes", () => {
    for (const k of ["+", "=", "Add"]) {
      expect(keyToAction(key({ key: k, ctrlKey: true }))).toEqual({
        command: "set_font_size",
        args: { delta: 1 },
      });
    }
    for (const k of ["-", "_", "Subtract"]) {
      expect(keyToAction(key({ key: k, ctrlKey: true }))).toEqual({
        command: "set_font_size",
        args: { delta: -1 },
      });
    }
  });

  it("maps ctrl 0 to a font size reset", () => {
    expect(keyToAction(key({ key: "0", ctrlKey: true }))).toEqual({
      command: "reset_font_size",
      args: {},
    });
  });

  it("maps escape and q to quit", () => {
    expect(keyToAction(key({ key: "Escape" }))?.command).toBe("quit");
    expect(keyToAction(key({ key: "Q" }))?.command).toBe("quit");
  });

  it("ignores unbound keys and ctrl+arrow", () => {
    expect(keyToAction(key({ key: "a" }))).toBeNull();
    expect(keyToAction(key({ key: "ArrowUp", ctrlKey: true }))).toBeNull();
  });
});
