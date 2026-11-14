import { describe, expect, test } from "vite-plus/test";
import { href, parseRoute } from "./routes";

describe("show routes", () => {
  test("links use the readable slug", () => {
    expect(href.show("arcane")).toBe("#/shows/arcane");
    expect(parseRoute(href.show("arcane"))).toEqual({ name: "show", ref: "arcane" });
  });

  test("non-Latin slugs survive the round trip", () => {
    expect(parseRoute(href.show("進撃の巨人"))).toEqual({ name: "show", ref: "進撃の巨人" });
  });

  test("legacy encoded-key links still parse to the internal key", () => {
    const legacy = "#/shows/11111111-1111-4111-8111-111111111111%00Arcane";
    expect(parseRoute(legacy)).toEqual({
      name: "show",
      ref: "11111111-1111-4111-8111-111111111111\u0000Arcane",
    });
  });

  test("the selected season round-trips as a query", () => {
    expect(href.show("arcane", "2")).toBe("#/shows/arcane?season=2");
    expect(parseRoute(href.show("arcane", "2"))).toEqual({
      name: "show",
      ref: "arcane",
      season: "2",
    });
    expect(parseRoute(href.show("arcane", "none"))).toEqual({
      name: "show",
      ref: "arcane",
      season: "none",
    });
  });

  test("malformed escapes are not found", () => {
    expect(parseRoute("#/shows/%E0%A4")).toEqual({ name: "not-found" });
  });
});

describe("settings and party routes", () => {
  test("settings sections round-trip", () => {
    expect(parseRoute(href.settings("users"))).toEqual({ name: "settings", section: "users" });
    expect(parseRoute(href.settings())).toEqual({ name: "settings", section: null });
  });

  test("old library links open the libraries section", () => {
    expect(parseRoute("#/libraries")).toEqual({ name: "settings", section: "libraries" });
  });

  test("unknown sections are not found", () => {
    expect(parseRoute("#/settings/billing")).toEqual({ name: "not-found" });
  });

  test("party links carry the party id", () => {
    expect(parseRoute(href.party("a/b"))).toEqual({ name: "party", id: "a/b" });
    expect(parseRoute(href.parties())).toEqual({ name: "parties" });
  });
});
