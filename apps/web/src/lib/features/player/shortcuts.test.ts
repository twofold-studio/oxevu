import { describe, expect, it } from "vite-plus/test";
import { playerShortcut } from "./shortcuts";

function key(key: string, kind = "background"): KeyboardEvent {
  return {
    key,
    target: {
      getAttribute: () => (kind === "seek" ? "Seek" : null),
      closest: (selector: string) => {
        if (selector.includes("textarea")) return kind === "menu" ? {} : null;
        if (selector.includes("input:not")) return kind === "text" ? {} : null;
        if (selector.includes("input[type=range]")) return kind === "volume" ? {} : null;
        return kind === "background" ? null : {};
      },
    },
  } as unknown as KeyboardEvent;
}

describe("arrow seeking", () => {
  it("uses five seconds by default and the configured interval", () => {
    expect(playerShortcut(key("ArrowLeft"))).toEqual({ type: "skip", seconds: -5 });
    expect(playerShortcut(key("ArrowRight"), 12)).toEqual({ type: "skip", seconds: 12 });
  });
  it("overrides tiny native seek steps and works after clicking a button", () => {
    for (const kind of ["seek", "button"]) {
      expect(playerShortcut(key("ArrowRight", kind), 7)).toEqual({ type: "skip", seconds: 7 });
    }
  });
  it("leaves volume, text entry and menu navigation alone", () => {
    for (const kind of ["volume", "text", "menu"]) {
      expect(playerShortcut(key("ArrowLeft", kind))).toBeNull();
    }
  });
});
