// ffprobe reports a demuxer list rather than a file type, e.g.
// "mov,mp4,m4a,3gp,3g2,mj2" or "matroska,webm". Map it to a readable label
// and a MIME type the browser can be asked about.

interface ContainerInfo {
  label: string;
  mime: string | null;
}

const KNOWN: [token: string, info: ContainerInfo][] = [
  ["mp4", { label: "MP4", mime: "video/mp4" }],
  ["mov", { label: "MP4", mime: "video/mp4" }],
  ["matroska", { label: "MKV", mime: "video/x-matroska" }],
  ["webm", { label: "WebM", mime: "video/webm" }],
  ["mkv", { label: "MKV", mime: "video/x-matroska" }],
  ["m4v", { label: "MP4", mime: "video/mp4" }],
  ["ogg", { label: "Ogg", mime: "video/ogg" }],
  ["avi", { label: "AVI", mime: "video/x-msvideo" }],
  ["mpegts", { label: "MPEG-TS", mime: "video/mp2t" }],
  ["ts", { label: "MPEG-TS", mime: "video/mp2t" }],
];

export function containerInfo(container: string | null): ContainerInfo | null {
  if (!container) return null;
  const tokens = container
    .toLowerCase()
    .split(/[,\s]+/)
    .filter(Boolean);
  // Prefer the first known token; "matroska,webm" files are usually MKV.
  for (const token of tokens) {
    const match = KNOWN.find(([known]) => known === token);
    if (match) return match[1];
  }
  return { label: tokens[0]?.toUpperCase() ?? container, mime: null };
}
