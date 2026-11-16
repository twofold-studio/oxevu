import { prefersReducedMotion } from "svelte/motion";
import { quintOut } from "svelte/easing";
import type { TransitionConfig } from "svelte/transition";

// JS side of the motion vocabulary in app.css, for Svelte `in:`/`out:`
// transitions (CSS keyframes cannot run on removal). Values follow the
// Gitadel source: a view change fades the old content out quickly, then the
// new content rises into place. Reduced motion skips both so the new content
// is there immediately; selection states never depend on these running.

/** Milliseconds; mirrors the `--motion-*` tokens in app.css. */
export const durations = {
  control: 180,
  exit: 110,
  enter: 280,
} as const;

/** Quintic ease-out matches the settling feel of CSS `--ease-settle`. */
export const settle = quintOut;

/**
 * Incoming view content: fade plus a 6px rise. The default delay lets an
 * outgoing `leave` finish first so the two views barely overlap; the wait is
 * shorter than a click and the content is reachable throughout.
 */
export function enter(
  _node: Element,
  {
    delay = durations.exit,
    duration = durations.enter,
    y = 6,
  }: { delay?: number; duration?: number; y?: number } = {},
): TransitionConfig {
  if (prefersReducedMotion.current) return { duration: 0 };
  return {
    delay,
    duration,
    easing: settle,
    css: (t, u) => `opacity: ${t}; transform: translateY(${u * y}px)`,
  };
}

/**
 * Outgoing view content: a short fade. The node is made inert immediately so
 * focus, clicks and the accessibility tree only ever see the new view, even
 * while several old views are still fading during rapid switching.
 */
export function leave(
  node: Element,
  { duration = durations.exit }: { duration?: number } = {},
): TransitionConfig {
  if (node instanceof HTMLElement) node.inert = true;
  if (prefersReducedMotion.current) return { duration: 0 };
  return { duration, css: (t) => `opacity: ${t}` };
}
