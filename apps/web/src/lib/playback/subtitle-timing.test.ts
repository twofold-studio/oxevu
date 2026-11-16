import { describe, expect, it } from "vite-plus/test";
import { offsetVtt } from "./subtitle-timing";

describe("subtitle timing", () => {
  it("delays or advances captions during direct play", () => {
    const text = "WEBVTT\n\n00:03.000 --> 00:05.000\nHello";
    expect(offsetVtt(text, 0, 1.5)).toContain("00:00:04.500 --> 00:00:06.500");
    expect(offsetVtt(text, 0, -1.5)).toContain("00:00:01.500 --> 00:00:03.500");
  });
  it("combines personal delay with the HLS seek offset without modifying text", () => {
    expect(offsetVtt("WEBVTT\n\n00:10:03.000 --> 00:10:05.000 align:end\nHello", 600, 2)).toContain(
      "00:00:05.000 --> 00:00:07.000 align:end\nHello",
    );
  });
  it("shifts absolute cues to an offset HLS session, keeping cue settings and text", () => {
    const vtt = "WEBVTT\n\n00:10:03.000 --> 00:10:05.500 align:start\nHello\n";
    expect(offsetVtt(vtt, 600)).toBe(
      "WEBVTT\n\n00:00:03.000 --> 00:00:05.500 align:start\nHello\n",
    );
  });
  it("drops completed cues instead of producing zero-length cues", () => {
    expect(
      offsetVtt("WEBVTT\n\n1\n00:01.000 --> 00:02.000\nOld\n\n2\n00:06.000 --> 00:07.000\nNew", 5),
    ).toBe("WEBVTT\n\n2\n00:00:01.000 --> 00:00:02.000\nNew");
  });
  it("clamps a cue crossing the seek point, and leaves direct play unchanged", () => {
    const vtt = "WEBVTT\n\n00:03.000 --> 00:06.000\nHello\n";
    expect(offsetVtt(vtt, 4)).toContain("00:00:00.000 --> 00:00:02.000");
    expect(offsetVtt(vtt, 0)).toBe(vtt);
  });
});
