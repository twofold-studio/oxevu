import { describe, expect, it, vi } from "vite-plus/test";
import { Autosave } from "./autosave";

const tick = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("Autosave", () => {
  it("does not write initial or invalid values and coalesces edits", async () => {
    const writes: number[] = [];
    const saver = new Autosave<number>(
      async (value) => {
        writes.push(value);
      },
      () => {},
      1,
    );
    saver.reset(1);
    saver.update(1);
    await saver.flush();
    expect(writes).toEqual([]);
    saver.update(2);
    saver.update(null);
    await saver.flush();
    expect(writes).toEqual([]);
    saver.update(3);
    saver.update(4);
    await saver.flush();
    expect(writes).toEqual([4]);
  });

  it("serializes pending writes without overwriting newer values", async () => {
    const writes: number[] = [];
    let finish!: () => void;
    const saver = new Autosave<number>(
      async (value) => {
        writes.push(value);
        if (value === 1)
          await new Promise<void>((resolve) => {
            finish = resolve;
          });
      },
      () => {},
      1000,
    );
    saver.update(1);
    const first = saver.flush();
    await tick();
    saver.update(2);
    saver.update(3);
    await saver.flush();
    expect(writes).toEqual([1]);
    finish();
    await first;
    expect(writes).toEqual([1, 3]);
  });

  it("reports failure and retries only after another edit", async () => {
    const status = vi.fn();
    let fail = true;
    const writes: number[] = [];
    const saver = new Autosave<number>(
      async (value) => {
        writes.push(value);
        if (fail) throw new Error("offline");
      },
      status,
      1000,
    );
    saver.update(1);
    await saver.flush();
    expect(status).toHaveBeenLastCalledWith("error", expect.any(Error));
    fail = false;
    saver.update(1);
    await saver.flush();
    expect(writes).toEqual([1, 1]);
    expect(status).toHaveBeenLastCalledWith("saved");
  });
});
