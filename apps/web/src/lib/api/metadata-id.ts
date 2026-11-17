import type { MetadataProvider } from "./metadata";

export function metadataId(provider: MetadataProvider, input: string, episode: boolean): string {
  let value = input.trim();
  if (value.startsWith("https://")) {
    const url = new URL(value);
    if (provider === "tmdb" && ["www.themoviedb.org", "themoviedb.org"].includes(url.hostname)) {
      const match = url.pathname.match(/^\/(movie|tv)\/(\d+)(?:-|\/|$)/);
      if (!match || match[1] !== (episode ? "tv" : "movie")) throw new Error("Choose a TMDB page for the correct media type.");
      value = match[2]!;
    } else if (provider === "omdb" && ["www.imdb.com", "imdb.com"].includes(url.hostname)) {
      value = url.pathname.match(/^\/title\/(tt\d+)(?:\/|$)/)?.[1] ?? "";
    } else throw new Error("Use a title URL from the selected provider's website.");
  }
  if (provider === "tmdb" && /^[1-9]\d{0,19}$/.test(value)) return value;
  if (provider === "omdb" && /^tt\d{1,18}$/.test(value)) return value;
  throw new Error(provider === "tmdb" ? "Enter the numeric TMDB ID or a TMDB title URL." : "Enter an IMDb ID such as tt1234567, or an IMDb title URL. OMDb uses IMDb IDs.");
}
