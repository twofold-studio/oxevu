import type { Item } from "$lib/api/types";
import { episodeName, episodeNumberLabel } from "$lib/media/grouping";

/** One-line label for pickers and party lists. */
export function itemLabel(item: Item): string {
  if (item.kind !== "episode") return item.title;
  return [item.show_title, episodeNumberLabel(item), episodeName(item)].filter(Boolean).join(" · ");
}

/** Case-insensitive match on every word, capped for long catalogs. */
export function searchItems(items: Item[], query: string, limit = 50): Item[] {
  const words = query.toLowerCase().split(/\s+/).filter(Boolean);
  const matches = words.length
    ? items.filter((item) => {
        const label = itemLabel(item).toLowerCase();
        return words.every((word) => label.includes(word));
      })
    : items;
  return matches.slice(0, limit);
}
