import HouseIcon from "@lucide/svelte/icons/house";
import FilmIcon from "@lucide/svelte/icons/film";
import TvIcon from "@lucide/svelte/icons/tv";
import UsersRoundIcon from "@lucide/svelte/icons/users-round";
import SettingsIcon from "@lucide/svelte/icons/settings";
import type { Component } from "svelte";
import type { User } from "$lib/api/types";
import { canJoinParties } from "$lib/state/access";
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
const parties: NavItem = {
  label: "Parties",
  href: href.parties(),
  icon: UsersRoundIcon,
  active: (r) => r.name === "parties" || r.name === "party",
};
const settings: NavItem = {
  label: "Settings",
  href: href.settings(),
  icon: SettingsIcon,
  active: (r) => r.name === "settings",
};

/** Role-aware hints; the server still authorizes every page's requests. */
export function browseNav(user: User | null): NavItem[] {
  return canJoinParties(user) ? [home, movies, shows, parties] : [home, movies, shows];
}

export const manageNav: NavItem[] = [settings];
