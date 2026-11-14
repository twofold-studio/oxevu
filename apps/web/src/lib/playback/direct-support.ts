import type { Item } from "$lib/api/types";
import { checkCompatibility } from "$lib/media/compatibility";

/**
 * What the client tells the server in `direct_play_supported`. "Uncertain"
 * (e.g. Matroska with decodable codecs) is reported as supported: direct play
 * is preferred when it works, and a real native failure switches to a
 * converted stream explicitly (see PlaybackEngine.handleMediaError).
 */
export function directPlaySupported(item: Item): boolean {
  const { verdict } = checkCompatibility(item);
  return verdict === "likely" || verdict === "uncertain";
}
