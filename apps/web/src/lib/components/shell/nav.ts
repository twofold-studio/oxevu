import HouseIcon from "@lucide/svelte/icons/house";
import FilmIcon from "@lucide/svelte/icons/film";
import TvIcon from "@lucide/svelte/icons/tv";
import CompassIcon from "@lucide/svelte/icons/compass";
import DownloadIcon from "@lucide/svelte/icons/download";
import type { Component } from "svelte";
import type { User } from "$lib/api/types";
import { href, type Route } from "$lib/state/router.svelte";

export interface NavItem {
  label: string;
  href: string;
  icon: Component;
  active: (route: Route) => boolean;
}

const home: NavItem = {
  label: "Home",
  href: href.home(),
  icon: HouseIcon,
  active: (r) => r.name === "home",
};
const movies: NavItem = {
  label: "Movies",
  href: href.movies(),
  icon: FilmIcon,
  active: (r) => r.name === "movies",
};
const shows: NavItem = {
  label: "Shows",
  href: href.shows(),
  icon: TvIcon,
  active: (r) => r.name === "shows" || r.name === "show",
};
/** Role-aware hints; the server still authorizes every page's requests. */
export function browseNav(user: User | null): NavItem[] {
  const items = [home, movies, shows];
  items.push({
    label: "Discover",
    href: href.discover(),
    icon: CompassIcon,
    active: (r) => r.name === "discover" || r.name === "discover-title",
  });
  if (user?.is_admin || user?.acquisition?.manage_downloads)
    items.push({
      label: "Activity",
      href: href.activity(),
      icon: DownloadIcon,
      active: (r) => r.name === "activity",
    });
  return items;
}
// Settings are opened from the account menu and bring their own navigation.
