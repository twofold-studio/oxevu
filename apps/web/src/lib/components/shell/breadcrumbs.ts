import type { Route } from "$lib/state/router.svelte";
import { href } from "$lib/state/router.svelte";
import type { Item } from "$lib/api/types";
import type { Show } from "$lib/media/grouping";
import { showKey } from "$lib/media/grouping";

export interface Crumb {
  label: string;
  href?: string;
}

interface Lookup {
  find: (id: string) => Item | undefined;
  shows: Show[];
}

export function breadcrumbs(route: Route, catalog: Lookup): Crumb[] {
  switch (route.name) {
    case "home":
      return [{ label: "Home" }];
    case "movies":
      return [{ label: "Movies" }];
    case "shows":
      return [{ label: "Shows" }];
    case "show": {
      const show = catalog.shows.find((s) => s.key === route.key);
      return [{ label: "Shows", href: href.shows() }, { label: show?.title ?? "Series" }];
    }
    case "item": {
      const item = catalog.find(route.id);
      if (!item) return [{ label: "Title" }];
      if (item.kind === "movie")
        return [{ label: "Movies", href: href.movies() }, { label: item.title }];
      return [
        { label: "Shows", href: href.shows() },
        { label: item.show_title ?? "Series", href: href.show(showKey(item)) },
        { label: item.title },
      ];
    }
    case "search":
      return [{ label: "Search" }];
    case "libraries":
      return [{ label: "Libraries" }];
    default:
      return [{ label: "Not found" }];
  }
}
