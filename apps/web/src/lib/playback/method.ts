import type { PlaybackMethod, PlaybackMode } from "$lib/api/playback-types";

export const methodLabel: Record<PlaybackMethod, string> = {
  direct: "Original file",
  remux: "Repackaged",
  cpu: "Converting · CPU",
  nvenc: "Converting · NVIDIA",
};

export const methodDetail: Record<PlaybackMethod, string> = {
  direct: "The browser plays the original file without conversion.",
  remux: "The video is copied into a browser-friendly stream without re-encoding.",
  cpu: "The server converts the video to H.264/AAC with the CPU.",
  nvenc: "The server converts the video to H.264/AAC with the NVIDIA encoder.",
};

export const modeLabel: Record<PlaybackMode, string> = {
  auto: "Automatic",
  direct: "Original file",
  transcode: "Convert",
};
