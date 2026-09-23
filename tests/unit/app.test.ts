import { describe, expect, it, vi, beforeEach } from "vitest";
import { mount, unmount } from "svelte";

const invoke = vi.fn();
const listen = vi.fn().mockResolvedValue(() => {});
vi.mock("@tauri-apps/api/core", () => ({ invoke }));
vi.mock("@tauri-apps/api/event", () => ({ listen }));

// jsdom has no ResizeObserver; App.svelte needs one on mount.
vi.stubGlobal(
  "ResizeObserver",
  class {
    observe() {}
    disconnect() {}
  },
);

const { default: App } = await import("../../src/App.svelte");

function state(overrides = {}) {
  return {
    displayName: "",
    shortcuts: [],
    fontSize: 14,
    ...overrides,
  };
}

describe("App", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    invoke.mockReset();
  });

  // Spec: with no shortcuts to show the window is hidden — nothing may be
  // rendered, or the window floats as a small empty blob over other apps.
  it("renders nothing when there are no shortcuts", async () => {
    invoke.mockResolvedValue(state());
    const app = mount(App, { target: document.body });
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("get_state"));
    await Promise.resolve();

    expect(document.querySelector(".frame")).toBeNull();
    await unmount(app);
  });

  it("renders the shortcut table when shortcuts exist", async () => {
    invoke.mockResolvedValue(
      state({
        displayName: "Google Chrome",
        shortcuts: [{ action: "New tab", keys: "Ctrl+T" }],
      }),
    );
    const app = mount(App, { target: document.body });
    await vi.waitFor(() =>
      expect(document.querySelector(".action")?.textContent).toBe("New tab"),
    );

    expect(document.querySelector(".title")?.textContent).toBe("Google Chrome");
    expect(document.querySelector(".action")?.textContent).toBe("New tab");
    expect(document.querySelector("kbd")?.textContent).toBe("Ctrl+T");
    await unmount(app);
  });
});
