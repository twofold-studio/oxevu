// Attaches a stream URL to a <video>. Direct streams use `src`. HLS uses
// hls.js wherever Media Source Extensions exist (its error recovery and
// buffering are consistent across browsers, unlike Chrome's newer native
// HLS); native HLS is the fallback for browsers without MSE, such as older
// iPhone Safari. hls.js loads on demand.

export interface AttachedStream {
  detach(): void;
}

export interface StreamFailure {
  /** Fatal HLS network/media error description. */
  message: string;
  /** HTTP status of the failed request, when there was one. */
  status: number | null;
}

const HLS_MIME = "application/vnd.apple.mpegurl";

export function nativeHls(video: HTMLVideoElement): boolean {
  return video.canPlayType(HLS_MIME) !== "";
}

function attachSrc(video: HTMLVideoElement, url: string): AttachedStream {
  video.src = url;
  return {
    detach() {
      video.pause();
      video.removeAttribute("src");
      video.load();
    },
  };
}

type HlsModule = typeof import("hls.js");
let hlsModule: Promise<HlsModule> | null = null;
let loadedHls: HlsModule | null = null;

/**
 * Load hls.js when this browser needs it. Called before attaching so the
 * attach itself is synchronous: callers check for superseded loads after the
 * await and never have two players racing for one <video>.
 */
const hasMse = () =>
  typeof window !== "undefined" && ("MediaSource" in window || "ManagedMediaSource" in window);

export async function prepareHls(video: HTMLVideoElement): Promise<void> {
  if (loadedHls || (!hasMse() && nativeHls(video))) return;
  hlsModule ??= import("hls.js");
  try {
    loadedHls = await hlsModule;
  } catch (error) {
    hlsModule = null;
    throw error;
  }
}

export function attachStream(
  video: HTMLVideoElement,
  url: string,
  hls: boolean,
  onFailure: (failure: StreamFailure) => void,
): AttachedStream {
  if (!hls) return attachSrc(video, url);
  const Hls = loadedHls?.default;
  if (!Hls?.isSupported()) {
    if (nativeHls(video)) return attachSrc(video, url);
    throw new Error("This browser supports neither native HLS nor Media Source Extensions.");
  }
  const player = new Hls({
    // Same-origin cookie auth; never put credentials in URLs.
    xhrSetup: (xhr) => {
      xhr.withCredentials = true;
    },
    // The server produces the stream from the session start; there is no
    // live edge to chase.
    startPosition: 0,
    backBufferLength: 60,
  });
  let recovered = false;
  player.on(Hls.Events.ERROR, (_event, data) => {
    if (!data.fatal) return;
    // One media-error recovery attempt (e.g. a decoder hiccup), then report.
    if (data.type === Hls.ErrorTypes.MEDIA_ERROR && !recovered) {
      recovered = true;
      player.recoverMediaError();
      return;
    }
    const status = data.response?.code || null;
    onFailure({
      status,
      message: status
        ? `The stream request failed (${status}).`
        : `The stream failed: ${data.details}.`,
    });
  });
  player.loadSource(url);
  player.attachMedia(video);
  return {
    detach() {
      player.destroy();
      video.removeAttribute("src");
      video.load();
    },
  };
}
