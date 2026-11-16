import { describe, expect, test } from "vite-plus/test";
import { canCreateParties, canJoinParties, settingsSectionsFor } from "./access";

describe("access hints", () => {
  test("only administrators see server settings", () => {
    expect(settingsSectionsFor({ is_admin: true })).toEqual([
      "libraries",
      "playback",
      "integrations",
      "acquisition",
      "users",
      "account",
    ]);
    expect(settingsSectionsFor({ is_admin: false })).toEqual(["account"]);
    expect(settingsSectionsFor(null)).toEqual(["account"]);
  });

  test("watch-party access follows syncplay_access", () => {
    expect(canJoinParties({ syncplay_access: "join" })).toBe(true);
    expect(canCreateParties({ syncplay_access: "join" })).toBe(false);
    expect(canCreateParties({ syncplay_access: "create_join" })).toBe(true);
    expect(canJoinParties({ syncplay_access: "disabled" })).toBe(false);
    expect(canJoinParties({})).toBe(false);
  });
});
