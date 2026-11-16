import { describe, expect, it, vi } from "vite-plus/test";
import type { Download, Release, ReleaseResults } from "$lib/api/acquisition-types";
import { ReleaseSearch } from "./release-search.svelte";
import { dedupeById, downloadFailure } from "./release-search";
import { eta, transferError } from "./activity";

const result = (releases: Release[] = []): ReleaseResults => ({
  releases,
  errors: [],
  offset: 0,
  searched: { indexers: 1, failed: 0, took_ms: 40, hits: releases.length, sources: [] },
});
const release = { id: "one", title: "Fixture" } as Release;

describe("release search lifecycle", () => {
  it("renders duplicate IDs once and distinguishes completed empty results from idle", async () => {
    const warning = vi.spyOn(console, "warn").mockImplementation(() => {});
    expect(dedupeById([release, release])).toEqual([release]);
    const search = new ReleaseSearch(async () => result());
    expect(search.state.kind).toBe("idle");
    await search.search("default", 1);
    expect(search.state.kind).toBe("empty");
    warning.mockRestore();
  });
  it("keeps results during refresh and restores them on cancel, ignoring a late response", async () => {
    let complete: (value: ReleaseResults) => void = () => {};
    let count = 0;
    const search = new ReleaseSearch(async () =>
      ++count === 1 ? result([release]) : new Promise((resolve) => (complete = resolve)),
    );
    await search.search("default", 1);
    const pending = search.search("new", 1);
    expect(search.state.kind).toBe("searching");
    search.cancel();
    expect(search.state.kind).toBe("results");
    complete(result());
    await pending;
    expect(search.state.kind).toBe("results");
  });
  it("shows failed sources instead of an empty success", async () => {
    const search = new ReleaseSearch(async () => ({
      ...result(),
      errors: [{ indexer_id: "bad", indexer_name: "Fixture", error: "HTTP 429", retryable: true }],
      searched: { ...result().searched, failed: 1 },
    }));
    await search.search("default", 1);
    expect(search.state.kind).toBe("failed");
  });
  it("shows timeout feedback and aborts its request", async () => {
    vi.useFakeTimers();
    let signal: AbortSignal | null = null;
    const search = new ReleaseSearch(async (_, supplied) => {
      signal = supplied;
      return new Promise((_, reject) =>
        supplied.addEventListener("abort", () => reject(new Error("aborted"))),
      );
    });
    const pending = search.search("default", 1);
    await vi.advanceTimersByTimeAsync(90000);
    await pending;
    expect((signal as unknown as AbortSignal).aborted).toBe(true);
    expect(search.state).toMatchObject({
      kind: "failed",
      message: "The search did not finish. Some indexers did not respond.",
    });
    vi.useRealTimers();
  });
  it("never labels immediately failed or duplicate jobs as queued", () => {
    expect(downloadFailure({ state: "error", error: "indexer unavailable" } as Download)).toBe(
      "indexer unavailable",
    );
    expect(downloadFailure({ state: "duplicate", error: "Already managed" } as Download)).toBe(
      "Already managed",
    );
    expect(downloadFailure({ state: "queued" } as Download)).toBeNull();
  });
  it("only estimates ETA for active downloads and explains provider errors", () => {
    const job = {
      state: "downloading",
      stats: {
        total_bytes: 10 * 1024 * 1024,
        progress_bytes: 2 * 1024 * 1024,
        live: { download_speed: { mbps: 1 } },
      },
    } as Download;
    expect(eta(job)).toBe("0m 8s left");
    expect(eta({ ...job, state: "paused" })).toBeNull();
    expect(transferError("provider returned HTTP 301")).toContain("link Oxevu cannot fetch");
    expect(transferError("indexer returned HTTP 429")).toContain("rate limiting");
  });
});
