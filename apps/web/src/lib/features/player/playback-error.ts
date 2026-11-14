import type { Item } from "$lib/api/types";
import { containerInfo } from "$lib/media/container";

export interface PlaybackProblem {
  title: string;
  detail: string;
  /** Session expired: the right next step is signing in, not retrying. */
  signedOut?: boolean;
}

function formatList(item: Item): string {
  return [
    containerInfo(item.container)?.label,
    item.video_codec?.toUpperCase(),
    item.audio_codec?.toUpperCase(),
  ]
    .filter((v): v is string => Boolean(v))
    .join(", ");
}

/**
 * MediaError alone can't tell "unsupported format" from "server refused the
 * request", so ask the stream endpoint directly before blaming the format.
 */
export async function explainPlaybackError(
  error: MediaError | null,
  item: Item,
  streamUrl: string,
): Promise<PlaybackProblem> {
  let status: number | null = null;
  try {
    const response = await fetch(streamUrl, { method: "HEAD", credentials: "same-origin" });
    status = response.status;
  } catch {
    return {
      title: "The server stopped responding",
      detail:
        "The video couldn't be loaded. Check that the Oxevu server is running, then try again.",
    };
  }

  if (status === 401) {
    return {
      title: "Your session has ended",
      detail: "Sign in again to keep watching.",
      signedOut: true,
    };
  }
  if (status === 403)
    return { title: "No access to this title", detail: "Your account can't play this item." };
  if (status === 404) {
    return {
      title: "The file is missing",
      detail:
        "The server couldn't find this file. Its library folder may be disconnected; rescan once it's back.",
    };
  }
  if (status !== null && status >= 500) {
    return {
      title: "The server couldn't read the file",
      detail: `The stream request failed (${status}).`,
    };
  }

  const formats = formatList(item);
  switch (error?.code) {
    case MediaError.MEDIA_ERR_NETWORK:
      return {
        title: "The stream stopped loading",
        detail: "The connection to the server was interrupted. Try again from where you stopped.",
      };
    case MediaError.MEDIA_ERR_DECODE:
      return {
        title: "This browser couldn't decode the file",
        detail: `Playback started but the browser failed to decode ${formats || "the video"}. Oxevu plays the original file and can't convert video yet.`,
      };
    default:
      return {
        title: "This browser can't play this file",
        detail: `${formats ? `It doesn't support ${formats}. ` : ""}Oxevu plays the original file and can't convert video yet. Try another browser, or keep using your current player for this title.`,
      };
  }
}
