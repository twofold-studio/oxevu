import { describe, expect, it } from "vite-plus/test";
import { SettingsDocument } from "./document";

interface Record {
  a: number;
  b: number;
}

/** A fake server that only accepts complete records, like PUT /settings. */
function server(initial: Record) {
  let stored = { ...initial };
  const writes: Record[] = [];
  const gates: (() => void)[] = [];
  let hold = false;
  return {
    writes,
    get stored() {
      return stored;
    },
    holdWrites() {
      hold = true;
    },
    release() {
      hold = false;
      for (const open of gates.splice(0)) open();
    },
    fetch: async () => ({ settings: { ...stored } }),
    save: async (value: Record) => {
      writes.push({ ...value });
      if (hold) await new Promise<void>((resolve) => gates.push(resolve));
      stored = { ...value };
      return { settings: { ...stored } };
    },
  };
}

describe("SettingsDocument", () => {
  it("serializes a delayed read with later edits so its snapshot cannot undo a save", async () => {
    const fake = server({ a: 1, b: 1 });
    let release = () => {};
    let hold = false;
    const doc = new SettingsDocument(
      async () => {
        const snapshot = await fake.fetch();
        if (hold)
          await new Promise<void>((resolve) => {
            release = resolve;
          });
        return snapshot;
      },
      fake.save,
      (e) => e.settings,
    );
    await doc.load();
    hold = true;
    const read = doc.load();
    await new Promise((resolve) => setTimeout(resolve, 0));
    const write = doc.patch({ a: 5 });
    await new Promise((resolve) => setTimeout(resolve, 0));
    expect(fake.writes).toEqual([]);
    release();
    await Promise.all([read, write]);
    await doc.patch({ b: 6 });
    expect(fake.stored).toEqual({ a: 5, b: 6 });
  });
  it("merges each page's fields onto the latest server copy", async () => {
    const fake = server({ a: 1, b: 1 });
    const doc = new SettingsDocument(fake.fetch, fake.save, (e) => e.settings);
    await doc.load();
    // Page A edits `a`; page B, opened from the same stale load, edits `b`.
    const first = doc.patch({ a: 2 });
    const second = doc.patch({ b: 3 });
    await Promise.all([first, second]);
    expect(fake.writes).toEqual([
      { a: 2, b: 1 },
      { a: 2, b: 3 },
    ]);
    expect(fake.stored).toEqual({ a: 2, b: 3 });
  });

  it("loads after queued writes so a new page never starts stale", async () => {
    const fake = server({ a: 1, b: 1 });
    const doc = new SettingsDocument(fake.fetch, fake.save, (e) => e.settings);
    await doc.load();
    fake.holdWrites();
    const write = doc.patch({ a: 5 });
    const loaded = doc.load();
    fake.release();
    await write;
    expect((await loaded).settings).toEqual({ a: 5, b: 1 });
  });

  it("keeps the queue moving after a failed write", async () => {
    const fake = server({ a: 1, b: 1 });
    let fail = true;
    const doc = new SettingsDocument(
      fake.fetch,
      async (value: Record) => {
        if (fail) {
          fail = false;
          throw new Error("refused");
        }
        return fake.save(value);
      },
      (e) => e.settings,
    );
    await doc.load();
    await expect(doc.patch({ a: 9 })).rejects.toThrow("refused");
    await doc.patch({ b: 4 });
    // The refused value is not resent with the next page's write.
    expect(fake.stored).toEqual({ a: 1, b: 4 });
  });
});
