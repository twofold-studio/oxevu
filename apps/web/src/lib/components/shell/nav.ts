import HouseIcon from "@lucide/svelte/icons/house";
import FilmIcon from "@lucide/svelte/icons/film";
import TvIcon from "@lucide/svelte/icons/tv";
import FolderCogIcon from "@lucide/svelte/icons/folder-cog";
import type { Component } from "svelte";
import { href, type Route } from "$lib/state/router.svelte";

export interface NavItem {
  label: string;
  href: string;
  icon: Component;
  active: (route: Route) => boolean;
}

export const browseNav: NavItem[] = [
  { label: "Home", href: href.home(), icon: HouseIcon, active: (r) => r.name === "home" },
  { label: "Movies", href: href.movies(), icon: FilmIcon, active: (r) => r.name === "movies" },
  {
    label: "Shows",
    href: href.shows(),
    icon: TvIcon,
    active: (r) => r.name === "shows" || r.name === "show",
  },
];

// The API has no role flag yet; the libraries page explains a 403 instead of
// hiding this entry.
export const manageNav: NavItem[] = [
  {
    label: "Libraries",
    href: href.libraries(),
    icon: FolderCogIcon,
    active: (r) => r.name === "libraries",
  },
];
