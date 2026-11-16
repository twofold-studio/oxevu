import { parseRoute, type Route } from "./routes";

export { href, parseRoute, type Route } from "./routes";

/** Pages that make sense to return to from settings; the player, party rooms and dead links do not. */
const returnable = (route: Route) =>
  !["settings", "watch", "party", "not-found"].includes(route.name);

class Router {
  route = $state<Route>(parseRoute(window.location.hash));
  /** Where "Back to library" leads from settings: the last browsing page. */
  returnTo = $state(returnable(this.route) ? window.location.hash || "#/" : "#/");

  constructor() {
    window.addEventListener("hashchange", () => {
      this.route = parseRoute(window.location.hash);
      if (returnable(this.route)) this.returnTo = window.location.hash || "#/";
      // New page, new scroll position, like a normal navigation.
      window.scrollTo({ top: 0 });
    });
  }

  go(target: string, replace = false) {
    if (replace) {
      history.replaceState(null, "", target);
      this.route = parseRoute(target);
      if (returnable(this.route)) this.returnTo = target;
    } else {
      window.location.hash = target;
    }
  }
}

export const router = new Router();
