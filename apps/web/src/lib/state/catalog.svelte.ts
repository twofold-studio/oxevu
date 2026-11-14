import { itemsApi } from "$lib/api/endpoints";
import { errorMessage } from "$lib/api/client";
import type { Item } from "$lib/api/types";
import { groupShows } from "$lib/media/grouping";

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
  inProgress = $derived(this.items.filter((item) => item.position_seconds > 0 && !item.completed));

  private pending: Promise<void> | null = null;

  /** Loads once; later calls reuse the result unless `force` is set. */
  ensure(force = false): Promise<void> {
    if (this.pending) return this.pending;
    if (this.status === "ready" && !force) return Promise.resolve();
    this.pending = this.fetch().finally(() => (this.pending = null));
    return this.pending;
  }

  private async fetch() {
    if (this.status !== "ready") this.status = "loading";
    this.error = null;
    try {
      const { items } = await itemsApi.list();
      this.items = items;
      this.status = "ready";
    } catch (error) {
      this.error = errorMessage(error);
      this.status = "error";
    }
  }

  find(id: string): Item | undefined {
    return this.items.find((item) => item.id === id);
  }

  /** Keep the cached copy in step with progress written by the player. */
  patch(id: string, changes: Partial<Item>) {
    const index = this.items.findIndex((item) => item.id === id);
    if (index >= 0) this.items[index] = { ...this.items[index], ...changes };
  }

  reset() {
    this.items = [];
    this.status = "idle";
    this.error = null;
  }
}

export const catalog = new Catalog();
