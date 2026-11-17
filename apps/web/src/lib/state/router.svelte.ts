import { parseRoute, type Route } from "./routes";
import { tick } from "svelte";
import Lenis from "lenis";

export { href, parseRoute, type Route } from "./routes";

/** Pages that make sense to return to from settings; the player, party rooms and dead links do not. */
const returnable = (route: Route) =>
  !["settings", "watch", "party", "not-found"].includes(route.name);

class Router {
  private readonly libraryScroll = new Map<string, number>();
  private navigation = 0;
  private lenis: Lenis | null = null;
  route = $state<Route>(parseRoute(window.location.hash));
  /** Where "Back to library" leads from settings: the last browsing page. */
  returnTo = $state(returnable(this.route) ? window.location.hash || "#/" : "#/");

  constructor() {
    history.scrollRestoration = "manual";
    window.addEventListener("hashchange", (event) => {
      const previousHash = new URL(event.oldURL).hash || "#/";
      if (this.route.name === "movies" || this.route.name === "shows" || this.route.name === "show") {
        this.libraryScroll.set(previousHash, window.scrollY);
      }
      this.route = parseRoute(window.location.hash);
      if (returnable(this.route)) this.returnTo = window.location.hash || "#/";
      const top = this.libraryScroll.get(window.location.hash) ?? 0;
      const navigation = ++this.navigation;
      void tick().then(() => {
        if (navigation !== this.navigation) return;
        if (this.lenis) {
          this.lenis.resize();
          this.lenis.scrollTo(top, { immediate: true, force: true });
        } else {
          window.scrollTo({ top, behavior: "instant" });
        }
      });
    });
  }

  mountSmoothScroll() {
    const lenis = new Lenis({
      autoRaf: true,
      allowNestedScroll: true,
      respectReducedMotion: true,
      stopInertiaOnNavigate: true,
      prevent: (element) => element.matches('[role="dialog"], [role="alertdialog"], [role="menu"], [role="listbox"]'),
    });
    this.lenis = lenis;
    return () => {
      lenis.destroy();
      this.lenis = null;
    };
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
