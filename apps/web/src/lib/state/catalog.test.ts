import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { catalog, catalogVisibleRefresh, fileAvailable, itemPlayable } from "./catalog.svelte";
import { itemsApi } from "$lib/api/endpoints";
import type { Item } from "$lib/api/types";

function item(overrides: Partial<Item> = {}): Item {
  return {
    id: "test",
    library_id: "lib",
    title: "Test",
    kind: "movie",
    show_title: null,
    season_number: null,
    episode_number: null,
    duration_seconds: 100,
    container: "mkv",
    video_codec: "h264",
    audio_codec: "aac",
    position_seconds: 0,
    completed: false,
    ...overrides,
  };
}

describe("fileAvailable", () => {
  it("returns true when file_available is undefined", () => {
    expect(fileAvailable(item())).toBe(true);
  });
  it("returns true when file_available is true", () => {
    expect(fileAvailable(item({ file_available: true }))).toBe(true);
  });
  it("returns false when file_available is false", () => {
    expect(fileAvailable(item({ file_available: false }))).toBe(false);
  });
});

describe("itemPlayable", () => {
  it("returns true for local file", () => {
    expect(itemPlayable(item({ file_available: true }))).toBe(true);
  });
  it("returns true for playable download without local file", () => {
    expect(
      itemPlayable(
        item({
          file_available: false,
          download: {
            id: "d1",
            file_index: 0,
            state: "downloading",
            bytes_received: 1024,
            size_bytes: 2048,
            playable: true,
          },
        }),
      ),
    ).toBe(true);
  });
  it("returns false for non-playable download without local file", () => {
    expect(
      itemPlayable(
        item({
          file_available: false,
          download: {
            id: "d1",
            file_index: 0,
            state: "paused",
            bytes_received: 0,
            size_bytes: 2048,
            playable: false,
          },
        }),
      ),
    ).toBe(false);
  });
  it("returns false for identity-changed download", () => {
    expect(
      itemPlayable(
        item({
          file_available: false,
          download: {
            id: "d1",
            file_index: 0,
            state: "identity_changed",
            bytes_received: 1024,
            size_bytes: 2048,
            playable: true,
          },
        }),
      ),
    ).toBe(false);
  });
  it("returns false for no file and no download", () => {
    expect(itemPlayable(item({ file_available: false }))).toBe(false);
  });
});

describe("visible catalog refresh", () => {
  let releases: (() => void)[] = [];
  afterEach(() => {
    releases.forEach((release) => release());
    releases = [];
    catalog.reset();
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  });

  it("shares requests, never overlaps and drops a late response after the final consumer leaves", async () => {
    vi.useFakeTimers();
    vi.stubGlobal("document", Object.assign(new EventTarget(), { hidden: false }));
    let resolve!: (result: { items: Item[] }) => void;
    const list = vi.spyOn(itemsApi, "list").mockImplementation(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    releases = [catalogVisibleRefresh(), catalogVisibleRefresh()];
    await vi.advanceTimersByTimeAsync(30_000);
    expect(list).toHaveBeenCalledTimes(1);
    const signal = list.mock.calls[0][0];
    releases[0]();
    expect(signal?.aborted).toBe(false);
    releases[1]();
    expect(signal?.aborted).toBe(true);
    resolve({ items: [item()] });
    await vi.advanceTimersByTimeAsync(30_000);
    expect(catalog.items).toEqual([]);
    expect(list).toHaveBeenCalledTimes(1);
  });

  it("pauses while hidden and refreshes when visible again", async () => {
    vi.useFakeTimers();
    const visibility = Object.assign(new EventTarget(), { hidden: false });
    vi.stubGlobal("document", visibility);
    const list = vi.spyOn(itemsApi, "list").mockResolvedValue({ items: [item()] });
    releases = [catalogVisibleRefresh()];
    await vi.advanceTimersByTimeAsync(0);
    visibility.hidden = true;
    visibility.dispatchEvent(new Event("visibilitychange"));
    await vi.advanceTimersByTimeAsync(30_000);
    expect(list).toHaveBeenCalledTimes(1);
    visibility.hidden = false;
    visibility.dispatchEvent(new Event("visibilitychange"));
    await vi.advanceTimersByTimeAsync(0);
    expect(list).toHaveBeenCalledTimes(2);
  });

  it("does not repopulate a signed-out catalog from an old request", async () => {
    let resolve!: (result: { items: Item[] }) => void;
    vi.spyOn(itemsApi, "list").mockImplementation(
      () =>
        new Promise((done) => {
          resolve = done;
        }),
    );
    const pending = catalog.ensure(true);
    catalog.reset();
    resolve({ items: [item()] });
    await pending;
    expect(catalog.items).toEqual([]);
    expect(catalog.status).toBe("idle");
  });
});
