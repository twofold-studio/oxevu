import { request, errorMessage } from "$lib/api/client";
import { offsetVtt } from "./subtitle-timing";
import { readSubtitlePreference, saveSubtitlePreference } from "./personal-preferences";

export interface SubtitleTrack {
  id: string;
  label: string;
  language: string | null;
  codec: string;
  default: boolean;
  forced: boolean;
  supported: boolean;
  url: string | null;
}

export class Subtitles {
  tracks = $state<SubtitleTrack[]>([]);
  selected = $state("off");
  searchOpen = $state(false);
  error = $state<string | null>(null);
  loading = $state(false);
  private video: HTMLVideoElement | null = null;
  private offset = 0;
  private element: HTMLTrackElement | null = null;
  private objectUrl: string | null = null;
  private abort: AbortController | null = null;
  // Cache only the latest file: switching tracks cannot grow memory indefinitely.
  private cached: { id: string; text: string } | null = null;

  async load(itemId: string, chooseDefault = true) {
    try {
      const { tracks } = await request<{ tracks: SubtitleTrack[] }>(
        `/items/${encodeURIComponent(itemId)}/subtitles`,
      );
      this.tracks = tracks;
      const preference = readSubtitlePreference();
      const preferred = preference
        ? "off" in preference
          ? null
          : tracks.find(
              (track) =>
                track.supported &&
                (preference.language
                  ? track.language === preference.language
                  : track.label === preference.label),
            )
        : (tracks.find((t) => t.supported && t.forced) ??
          tracks.find((t) => t.supported && t.default));
      if (chooseDefault) this.select(preferred?.id ?? "off", false);
      return true;
    } catch (error) {
      this.error = errorMessage(error);
      return false;
    }
  }

  attach(video: HTMLVideoElement, offset: number) {
    this.video = video;
    this.offset = offset;
    void this.render();
  }

  select(id: string, remember = true) {
    this.selected = id;
    if (remember) {
      const track = this.tracks.find((track) => track.id === id);
      if (id === "off") saveSubtitlePreference({ off: true });
      else if (track) saveSubtitlePreference({ language: track.language, label: track.label });
    }
    void this.render();
  }

  dispose() {
    this.abort?.abort();
    this.clear();
    this.video = null;
  }

  private clear() {
    this.element?.remove();
    this.element = null;
    if (this.objectUrl) URL.revokeObjectURL(this.objectUrl);
    this.objectUrl = null;
  }

  private async render() {
    this.abort?.abort();
    this.clear();
    this.error = null;
    this.loading = false;
    const track = this.tracks.find((t) => t.id === this.selected);
    const video = this.video;
    if (!video || !track?.supported || !track.url) return;
    // Only the authenticated item endpoint may supply subtitle data.
    if (!/^\/api\/v1\/items\/[^/]+\/subtitles\/[^/?#]+$/.test(track.url)) {
      this.error = "Invalid subtitle address.";
      return;
    }
    const abort = new AbortController();
    this.abort = abort;
    this.loading = true;
    try {
      let text = this.cached?.id === track.id ? this.cached.text : undefined;
      if (text === undefined) {
        const response = await fetch(track.url, {
          credentials: "same-origin",
          signal: abort.signal,
        });
        if (!response.ok) throw new Error(`Couldn't load subtitles (${response.status}).`);
        if (Number(response.headers.get("content-length")) > 8_000_000)
          throw new Error("Subtitle file is too large.");
        text = await response.text();
        if (text.length > 8_000_000) throw new Error("Subtitle file is too large.");
        this.cached = { id: track.id, text };
      }
      if (abort.signal.aborted) return;
      this.objectUrl = URL.createObjectURL(
        new Blob([offsetVtt(text, this.offset)], { type: "text/vtt" }),
      );
      const element = document.createElement("track");
      element.kind = "subtitles";
      element.label = track.label;
      element.srclang = track.language ?? "und";
      element.src = this.objectUrl;
      element.default = true;
      element.addEventListener(
        "load",
        () => {
          element.track.mode = "showing";
        },
        { once: true },
      );
      element.addEventListener(
        "error",
        () => {
          if (this.element === element) this.error = "This subtitle file could not be displayed.";
        },
        { once: true },
      );
      this.element = element;
      video.append(element);
      element.track.mode = "showing";
    } catch (error) {
      if (!abort.signal.aborted) this.error = errorMessage(error);
    } finally {
      if (!abort.signal.aborted) this.loading = false;
    }
  }
}
