export type PlayerAction =
  | { type: "toggle" }
  | { type: "skip"; seconds: number }
  | { type: "volume"; delta: number }
  | { type: "mute" }
  | { type: "fullscreen" };

/**
 * Maps keys to player actions. Space and arrows are left alone while a
 * control has focus so buttons and sliders keep their native behaviour.
 */
export function playerShortcut(event: KeyboardEvent): PlayerAction | null {
  if (event.metaKey || event.ctrlKey || event.altKey) return null;
  const target = event.target as HTMLElement | null;
  const onControl = target?.closest("button, input, a, [role=slider]") !== null && target !== null;

  switch (event.key) {
    case "k":
    case "K":
      return { type: "toggle" };
    case " ":
      return onControl ? null : { type: "toggle" };
    case "ArrowLeft":
      return onControl ? null : { type: "skip", seconds: -10 };
    case "ArrowRight":
      return onControl ? null : { type: "skip", seconds: 10 };
    case "j":
    case "J":
      return { type: "skip", seconds: -10 };
    case "l":
    case "L":
      return { type: "skip", seconds: 10 };
    case "ArrowUp":
      return onControl ? null : { type: "volume", delta: 0.1 };
    case "ArrowDown":
      return onControl ? null : { type: "volume", delta: -0.1 };
    case "m":
    case "M":
      return { type: "mute" };
    case "f":
    case "F":
      return { type: "fullscreen" };
    default:
      return null;
  }
}
