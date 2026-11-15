import { describe, expect, it } from "vite-plus/test";
import { mergeReadyRanges } from "./ready-ranges";

describe("ready ranges", () => {
  it("merges overlapping and touching buffers without filling gaps", () => {
    expect(
      mergeReadyRanges(
        [
          { start: 30, end: 50 },
          { start: 10, end: 40 },
          { start: 50, end: 60 },
          { start: 70, end: 80 },
        ],
        100,
      ),
    ).toEqual([
      { start: 10, end: 60 },
      { start: 70, end: 80 },
    ]);
  });
  it("clamps to the timeline and ignores invalid ranges", () => {
    expect(
      mergeReadyRanges(
        [
          { start: -10, end: 5 },
          { start: 90, end: 120 },
          { start: NaN, end: 50 },
          { start: 20, end: 10 },
        ],
        100,
      ),
    ).toEqual([
      { start: 0, end: 5 },
      { start: 90, end: 100 },
    ]);
  });
});
