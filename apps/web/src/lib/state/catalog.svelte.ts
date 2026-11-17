import { itemsApi } from "$lib/api/endpoints";
import { errorMessage } from "$lib/api/client";
import type { Item } from "$lib/api/types";
import { groupShows, showKey, type Show } from "$lib/media/grouping";
import { findShow } from "$lib/media/show-slug";
import { resumePosition, withWatchedStatus } from "$lib/media/progress";

/** True when the item's local file is present (or the field is missing on older servers). */
export const fileAvailable = (item: Item): boolean => item.file_available !== false;

/** True when the item can be played: either a local file or a playable download. */
export const itemPlayable = (item: Item): boolean =>
  fileAvailable(item) ||
  (item.download?.playable === true && item.download.state !== "identity_changed");

// One shared copy of the item list. Phase one returns every playable item in a
// single response, so browsing, search and grouping derive from it locally.
class Catalog {
  items = $state<Item[]>([]);
  status = $state<"idle" | "loading" | "ready" | "error">("idle");
  error = $state<string | null>(null);

  movies = $derived(
    this.items
      .filter((item) => item.kind === "movie")
      .toSorted((a, b) =>
        a.title.localeCompare(b.title, undefined, { sensitivity: "base", numeric: true }),
      ),
  );
  shows = $derived(groupShows(this.items));
  inProgress = $derived(
    this.items.filter((item) => resumePosition(item) !== null && itemPlayable(item)),
  );

  private pending: Promise<void> | null = null;
  private request: AbortController | null = null;
  private generation = 0;

  /** Loads once; later calls reuse the result unless `force` is set. */
  ensure(force = false, signal?: AbortSignal): Promise<void> {
    if (this.pending && !this.request?.signal.aborted) return this.pending;
    if (this.status === "ready" && !force) return Promise.resolve();
    const controller = new AbortController();
    const abort = () => controller.abort();
    signal?.addEventListener("abort", abort, { once: true });
    if (signal?.aborted) controller.abort();
    this.request = controller;
    this.pending = this.fetch(controller.signal, this.generation).finally(() => {
      signal?.removeEventListener("abort", abort);
      if (this.request === controller) {
        this.pending = null;
        this.request = null;
      }
    });
    return this.pending;
  }

  private async fetch(signal: AbortSignal, generation: number) {
    if (this.status !== "ready") this.status = "loading";
    this.error = null;
    try {
      const { items } = await itemsApi.list(signal);
      if (signal.aborted || generation !== this.generation) return;
      this.items = items.map(withWatchedStatus);
      this.status = "ready";
    } catch (error) {
      if (signal.aborted || generation !== this.generation) return;
      this.error = errorMessage(error);
      this.status = "error";
    }
  }

  find(id: string): Item | undefined {
    return this.items.find((item) => item.id === id);
  }

  /** Resolves a show URL segment (slug or legacy key). */
  findShow(ref: string): Show | undefined {
    return findShow(this.shows, ref);
  }

  /** The series an episode belongs to, for links back to it. */
  showOf(item: Item): Show | undefined {
    if (item.kind !== "episode") return undefined;
    const key = showKey(item);
    return this.shows.find((show) => show.key === key);
  }

  /** Keep the cached copy in step with progress written by the player. */
  patch(id: string, changes: Partial<Item>) {
    const index = this.items.findIndex((item) => item.id === id);
    if (index >= 0) this.items[index] = withWatchedStatus({ ...this.items[index], ...changes });
  }

  reset() {
    this.generation++;
    this.request?.abort();
    this.pending = null;
    this.request = null;
    this.items = [];
    this.status = "idle";
    this.error = null;
  }
}

export const catalog = new Catalog();

/**
 * Bounded visibility-aware catalog refresh. Returns a cleanup function.
 * No overlapping requests; pauses when the tab is hidden. The existing
 * `ensure(true)` deduplicates concurrent calls, so an interval tick during
 * an in-flight request is harmless.
 */
let refreshOwners = 0;
let refreshTimer: ReturnType<typeof setTimeout> | undefined;
let refreshRequest: AbortController | null = null;

async function refreshVisibleCatalog() {
  if (!refreshOwners || document.hidden || refreshRequest) return;
  const request = new AbortController();
  refreshRequest = request;
  try {
    await catalog.ensure(true, request.signal);
  } finally {
    if (refreshRequest === request) {
      refreshRequest = null;
      if (refreshOwners && !document.hidden)
        refreshTimer = setTimeout(refreshVisibleCatalog, 10_000);
    }
  }
}

function refreshVisibility() {
  clearTimeout(refreshTimer);
  refreshRequest?.abort();
  refreshRequest = null;
  if (!document.hidden) void refreshVisibleCatalog();
}

/** One shared request loop for visible catalog consumers; no per-card timers. */
export function catalogVisibleRefresh(): () => void {
  if (++refreshOwners === 1) {
    document.addEventListener("visibilitychange", refreshVisibility);
    void refreshVisibleCatalog();
  }
  let released = false;
  return () => {
    if (released) return;
    released = true;
    if (--refreshOwners === 0) {
      document.removeEventListener("visibilitychange", refreshVisibility);
      clearTimeout(refreshTimer);
      refreshRequest?.abort();
      refreshRequest = null;
    }
  };
}
