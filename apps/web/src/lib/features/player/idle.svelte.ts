/** Tracks pointer/keyboard inactivity so player chrome can step aside. */
export function createIdle(timeoutMs: number) {
  let idle = $state(false);
  let timer: ReturnType<typeof setTimeout> | undefined;

  function poke() {
    idle = false;
    clearTimeout(timer);
    timer = setTimeout(() => (idle = true), timeoutMs);
  }

  return {
    get idle() {
      return idle;
    },
    poke,
    stop: () => clearTimeout(timer),
  };
}
