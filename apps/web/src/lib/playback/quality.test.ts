import { describe, expect, it } from "vite-plus/test";
import { AutomaticQuality, networkQuality } from "./quality";

describe("network quality", () => {
  it("reserves room for audio and traffic", () => {
    expect(networkQuality(6)?.video_bitrate_mbps).toBe(4);
    expect(networkQuality(2)?.max_height).toBe(360);
    expect(networkQuality(0)).toBeUndefined();
    expect(networkQuality(NaN)).toBeUndefined();
  });
  it("requires sustained weakness and a cooldown without upshifting", () => {
    const auto = new AutomaticQuality();
    expect(auto.sample(2, 8, 40_000)).toBeUndefined();
    expect(auto.sample(2, 8, 41_000)).toBeUndefined();
    expect(auto.sample(2, 8, 42_000)?.max_height).toBe(360);
    expect(auto.sample(0.5, 0.8, 43_000)).toBeUndefined();
    expect(auto.sample(20, 0.8, 80_000)).toBeUndefined();
  });
});
