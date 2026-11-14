import { ApiError } from "$lib/api/client";
import { SessionFailedError } from "./session-controller";

export interface PlaybackProblem {
  title: string;
  detail: string;
  /** Session expired: the right next step is signing in, not retrying. */
  signedOut?: boolean;
}

const signedOut: PlaybackProblem = {
  title: "Your session has ended",
  detail: "Sign in again to keep watching.",
  signedOut: true,
};

/** Problems creating or starting a server playback session. */
export function explainSessionError(error: unknown): PlaybackProblem {
  if (error instanceof SessionFailedError) {
    const reason = error.session?.reason;
    return {
      title: "The server couldn't prepare this stream",
      detail: reason ? `${error.message} (${reason})` : error.message,
    };
  }
  if (error instanceof ApiError) {
    if (error.unauthorized) return signedOut;
    if (error.forbidden) return { title: "Playback not allowed", detail: error.message };
    if (error.status === 404) {
      return {
        title: "This title is unavailable",
        detail: "It may have been removed by a rescan, or its library is offline.",
      };
    }
    if (error.status === 503 || error.status === 429) {
      return { title: "The server is busy", detail: error.message };
    }
    return { title: "Couldn't start playback", detail: error.message };
  }
  return {
    title: "Couldn't start playback",
    detail: error instanceof Error ? error.message : "Something went wrong.",
  };
}

/**
 * MediaError alone can't tell "unsupported format" from "server refused the
 * request", so ask the stream URL directly before blaming the format.
 */
export async function explainMediaError(
  error: MediaError | null,
  streamUrl: string,
  converted: boolean,
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
  if (status === 401) return signedOut;
  if (status === 403)
    return { title: "No access to this title", detail: "Your account can't play this item." };
  if (status === 404) {
    return converted
      ? {
          title: "The stream ended on the server",
          detail: "Try again to start a new stream from here.",
        }
      : {
          title: "The file is missing",
          detail:
            "The server couldn't find this file. Its library folder may be offline; rescan once it's back.",
        };
  }
  if (status !== null && status >= 500) {
    return {
      title: "The server couldn't read the file",
      detail: `The stream request failed (${status}).`,
    };
  }
  if (error?.code === MediaError.MEDIA_ERR_NETWORK) {
    return {
      title: "The stream stopped loading",
      detail: "The connection to the server was interrupted. Try again from where you stopped.",
    };
  }
  const said = error?.message ? ` The browser reported: ${error.message}` : "";
  return converted
    ? {
        title: "This browser couldn't play the converted stream",
        detail: `Try again, or try another browser.${said}`,
      }
    : {
        title: "This browser can't play the original file",
        detail: `Choose Convert in the player's playback menu if your account allows conversion.${said}`,
      };
}
