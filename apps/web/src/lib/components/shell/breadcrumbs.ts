import type { Route } from "$lib/state/router.svelte";
import { href } from "$lib/state/router.svelte";
import { isSettingsPage, settingsPageLabel } from "$lib/state/settings-pages";
import type { Item } from "$lib/api/types";
import { seasonKeyOf, type Show } from "$lib/media/grouping";

export interface Crumb {
  label: string;
  href?: string;
}

interface Lookup {
  find: (id: string) => Item | undefined;
  findShow: (ref: string) => Show | undefined;
  showOf: (item: Item) => Show | undefined;
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
      const show = catalog.findShow(route.ref);
      return [{ label: "Shows", href: href.shows() }, { label: show?.title ?? "Series" }];
    }
    case "item": {
      const item = catalog.find(route.id);
      if (!item) return [{ label: "Title" }];
      if (item.kind === "movie")
        return [{ label: "Movies", href: href.movies() }, { label: item.title }];
      const show = catalog.showOf(item);
      return [
        { label: "Shows", href: href.shows() },
        {
          label: show?.title ?? item.show_title ?? "Series",
          // Back to the show lands on this episode's season tab.
          href: show ? href.show(show.slug, seasonKeyOf(item.season_number)) : undefined,
        },
        { label: item.title },
      ];
    }
    case "search":
      return [{ label: "Search" }];
    case "settings":
      return route.target && isSettingsPage(route.target)
        ? [
            { label: "Settings", href: href.settings() },
            { label: settingsPageLabel(route.target).page },
          ]
        : [{ label: "Settings" }];
    case "parties":
      return [{ label: "Watch parties" }];
    case "discover":
      return [{ label: "Discover" }];
    case "discover-title":
      return [{ label: "Discover", href: href.discover("", route.kind) }, { label: "Title" }];
    case "requests":
      return [{ label: "Requests" }];
    case "wanted":
      return [{ label: "Wanted" }];
    case "activity":
      return [{ label: "Activity" }];
    default:
      return [{ label: "Not found" }];
  }
}
