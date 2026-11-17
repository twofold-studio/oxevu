import { describe, expect, it } from "vite-plus/test";
import { metadataId } from "./metadata-id";

describe("provider ID identification", () => {
  it("accepts raw IDs and provider title URLs", () => {
    expect(metadataId("tmdb", " 9476 ", false)).toBe("9476");
    expect(metadataId("tmdb", "https://www.themoviedb.org/tv/94605-arcane", true)).toBe("94605");
    expect(metadataId("omdb", "https://www.imdb.com/title/tt0183790/", false)).toBe("tt0183790");
    expect(metadataId("omdb", "tt0183790", false)).toBe("tt0183790");
  });
  it("rejects wrong media types, providers and untrusted URLs", () => {
    expect(() => metadataId("tmdb", "https://www.themoviedb.org/movie/9476", true)).toThrow("media type");
    for (const input of ["tt0183790", "0", "https://evil.test/movie/9476", "https://themoviedb.org.evil.test/movie/9476"]) {
      expect(() => metadataId("tmdb", input, false)).toThrow();
    }
    expect(() => metadataId("omdb", "9476", false)).toThrow("IMDb");
  });
});
