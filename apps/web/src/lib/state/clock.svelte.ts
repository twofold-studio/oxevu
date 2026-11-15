import { createSubscriber } from "svelte/reactivity";

// One shared timer for "Ends at" times. It only runs while something on
// screen reads `now()`, and stops when the last reader unmounts.
const TICK_MS = 30_000;
let current = Date.now();

const subscribe = createSubscriber((update) => {
  current = Date.now();
  const timer = setInterval(() => {
    current = Date.now();
    update();
  }, TICK_MS);
  return () => clearInterval(timer);
});

/** Current time in ms, refreshed every 30 seconds while observed. */
export function now(): number {
  subscribe();
  return current;
}
