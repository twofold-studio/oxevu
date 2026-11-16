import { describe, expect, it } from "vite-plus/test";
import type { Download } from "$lib/api/acquisition-types";
import { importMessage, validateTransferLimit } from "./activity";

describe("entry speed limits", () => {
  it("allows no cap and safe whole-number rates", () => {
    for (const value of [0, 16, 128, 1_000_000]) expect(validateTransferLimit(value)).toBeNull();
    for (const value of [undefined, NaN, -1, 1, 15, 16.5, 1_000_001])
      expect(validateTransferLimit(value)).not.toBeNull();
  });
});

describe("completed download import messages", () => {
  const job = {} as Download;
  it("explains the switch and missing prerequisites before worker warnings", () => {
    expect(importMessage(job, false)).toContain("imports are off");
    expect(
      importMessage(
        { ...job, import_blocker: "Choose a destination", import_warning: "Old error" },
        true,
      ),
    ).toBe("Choose a destination");
    expect(importMessage({ ...job, import_warning: "Library offline" }, true)).toBe(
      "Library offline",
    );
    expect(importMessage(job, true)).toContain("every 10 seconds");
  });
  it("shows a successful import even if the switch is later disabled", () => {
    expect(importMessage({ ...job, imported: true, import_warning: "Old error" }, false)).toContain(
      "Imported into the library",
    );
  });
});
