import { expect, test } from "vite-plus/test";
import type { Item } from "$lib/api/types";
import { fileFacts } from "./file-facts";

const episode = (id: string, extra: Partial<Item> = {}): Item => ({ id, library_id:"tv", title:id, kind:"episode", show_title:"Show", season_number:1, episode_number:1, duration_seconds:60, container:"matroska", video_codec:"h264", audio_codec:"aac", position_seconds:0, completed:false, size_bytes:1024, ...extra });

test("series size counts available files once and explains missing sizes", () => {
  const one = episode("one");
  const rows = fileFacts(one,[one,one,episode("two"),episode("gone",{file_available:false,size_bytes:9999}),episode("unknown",{size_bytes:undefined})]);
  expect(rows.find((r) => r.id === "size")?.value).toBe("At least 2 KiB");
  expect(rows.find((r) => r.id === "duration")?.value).toBe("3:00");
});
test("mixed series attributes are labelled instead of showing the first file as representative", () => {
  const one = episode("one");
  expect(fileFacts(one,[one,episode("two",{video_codec:"hevc"})]).find((r) => r.id === "video")?.value).toBe("Varies");
});
