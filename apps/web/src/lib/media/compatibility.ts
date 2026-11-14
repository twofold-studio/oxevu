import type { Item } from "$lib/api/types";
import { containerInfo } from "./container";

// Phase one has no transcoding, so we estimate up front whether this browser
// can decode the original file. canPlayType is a hint ("maybe"/"probably"),
// not a guarantee; the player still reports real failures.

// Representative RFC 6381 codec strings; profiles are typical, not exact.
const VIDEO_CODEC: Record<string, string> = {
  h264: "avc1.640028",
  avc: "avc1.640028",
  avc1: "avc1.640028",
  hevc: "hvc1.1.6.L120.90",
  h265: "hvc1.1.6.L120.90",
  av1: "av01.0.08M.08",
  vp9: "vp09.00.40.08",
  vp8: "vp8",
  theora: "theora",
};

const AUDIO_CODEC: Record<string, string> = {
  aac: "mp4a.40.2",
  mp3: "mp4a.69",
  opus: "opus",
  vorbis: "vorbis",
  flac: "flac",
  ac3: "ac-3",
  eac3: "ec-3",
};

export type Verdict = "likely" | "uncertain" | "unlikely" | "unknown";

export interface Compatibility {
  verdict: Verdict;
  /** Parts of the file this browser reports it can't decode. */
  blockers: string[];
}

let probe: HTMLVideoElement | null = null;
function canPlay(type: string): CanPlayTypeResult {
  probe ??= document.createElement("video");
  return probe.canPlayType(type);
}

const key = (value: string | null) => value?.toLowerCase().replace(/[^a-z0-9]/g, "") ?? null;

export function checkCompatibility(item: Item): Compatibility {
  const container = containerInfo(item.container);
  const mime = container?.mime;
  if (!container || !mime) return { verdict: "unknown", blockers: [] };

  const blockers: string[] = [];
  // Browsers rarely claim Matroska support even when they decode it, so only
  // its codecs are checked below.
  if (mime !== "video/x-matroska" && canPlay(mime) === "")
    blockers.push(`${container.label} container`);

  const video = key(item.video_codec);
  const audio = key(item.audio_codec);
  const results: CanPlayTypeResult[] = [];
  for (const [codec, table, label] of [
    [video, VIDEO_CODEC, item.video_codec],
    [audio, AUDIO_CODEC, item.audio_codec],
  ] as const) {
    if (!codec) continue;
    const id = table[codec];
    if (!id) continue;
    // Matroska support is reported inconsistently; probe codecs in MP4/WebM.
    const host =
      mime === "video/x-matroska"
        ? codec.startsWith("vp") || codec === "opus" || codec === "vorbis"
          ? "video/webm"
          : "video/mp4"
        : mime;
    const result = canPlay(`${host}; codecs="${id}"`);
    if (result === "")
      blockers.push(`${label?.toUpperCase()} ${table === VIDEO_CODEC ? "video" : "audio"}`);
    results.push(result);
  }

  if (blockers.length > 0) return { verdict: "unlikely", blockers };
  if (mime === "video/x-matroska") return { verdict: "uncertain", blockers };
  if (results.length > 0 && results.every((r) => r === "probably"))
    return { verdict: "likely", blockers };
  return {
    verdict: results.length > 0 || canPlay(mime) !== "" ? "uncertain" : "unknown",
    blockers,
  };
}
