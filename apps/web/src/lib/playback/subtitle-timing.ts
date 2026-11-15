const timestamp = /^(?:(\d+):)?(\d{2}):(\d{2})\.(\d{3})$/;
function seconds(value: string): number {
  const m = timestamp.exec(value);
  return m
    ? Number(m[1] ?? 0) * 3600 + Number(m[2]) * 60 + Number(m[3]) + Number(m[4]) / 1000
    : NaN;
}
function format(value: number): string {
  const ms = Math.round(Math.max(0, value) * 1000);
  return `${String(Math.floor(ms / 3600000)).padStart(2, "0")}:${String(Math.floor(ms / 60000) % 60).padStart(2, "0")}:${String(Math.floor(ms / 1000) % 60).padStart(2, "0")}.${String(ms % 1000).padStart(3, "0")}`;
}

/** VTT is in item time; HLS media starts at zero for each seek session. */
export function offsetVtt(vtt: string, offset: number): string {
  if (offset <= 0) return vtt;
  return vtt
    .split(/\r?\n\r?\n/)
    .flatMap((block) => {
      let expired = false;
      const shifted = block.replace(
        /^(\S+) --> (\S+)([^\r\n]*)\r?$/gm,
        (line, start: string, end: string, settings: string) => {
          const a = seconds(start),
            b = seconds(end);
          if (!Number.isFinite(a) || !Number.isFinite(b)) return line;
          expired = b <= offset;
          return `${format(a - offset)} --> ${format(b - offset)}${settings}`;
        },
      );
      return expired ? [] : [shifted];
    })
    .join("\n\n");
}
