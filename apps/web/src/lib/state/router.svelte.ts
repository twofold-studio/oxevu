import { parseRoute, type Route } from "./routes";

export { href, parseRoute, type Route } from "./routes";

class Router {
  route = $state<Route>(parseRoute(window.location.hash));

  constructor() {
    window.addEventListener("hashchange", () => {
      this.route = parseRoute(window.location.hash);
      // New page, new scroll position, like a normal navigation.
      window.scrollTo({ top: 0 });
    });
  }

  go(target: string, replace = false) {
    if (replace) {
      history.replaceState(null, "", target);
      this.route = parseRoute(target);
    } else {
      window.location.hash = target;
    }
  }
}

export const router = new Router();
