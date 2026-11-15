export type PlayerAction =
  | { type: "toggle" }
  | { type: "skip"; seconds: number }
  | { type: "volume"; delta: number }
  | { type: "mute" }
  | { type: "fullscreen" };

/**
 * Seek arrows work across playback controls; text entry, menus and volume
 * sliders retain their native keyboard behaviour.
 */
export function playerShortcut(event: KeyboardEvent, seekSeconds = 5): PlayerAction | null {
  if (event.metaKey || event.ctrlKey || event.altKey) return null;
  const target = event.target as HTMLElement | null;
  if (target?.closest("textarea, select, [contenteditable=true], [role=menu], [role=dialog]"))
    return null;
  if (target?.closest("input:not([type=range])")) return null;
  const onControl = target?.closest("button, input, a, [role=slider]") !== null && target !== null;
  const seekControl = target?.getAttribute("aria-label") === "Seek";
  const otherSlider = !seekControl && target?.closest("input[type=range], [role=slider]");

  switch (event.key) {
    case "k":
    case "K":
      return { type: "toggle" };
    case " ":
      return onControl ? null : { type: "toggle" };
    case "ArrowLeft":
      return otherSlider ? null : { type: "skip", seconds: -seekSeconds };
    case "ArrowRight":
      return otherSlider ? null : { type: "skip", seconds: seekSeconds };
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
