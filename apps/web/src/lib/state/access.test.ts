import { describe, expect, test } from "vite-plus/test";
import { canCreateParties, canJoinParties } from "./access";

// Settings page access is covered in settings-pages.test.ts.
describe("access hints", () => {
  test("watch-party access follows syncplay_access", () => {
    expect(canJoinParties({ syncplay_access: "join" })).toBe(true);
    expect(canCreateParties({ syncplay_access: "join" })).toBe(false);
    expect(canCreateParties({ syncplay_access: "create_join" })).toBe(true);
    expect(canJoinParties({ syncplay_access: "disabled" })).toBe(false);
    expect(canJoinParties({})).toBe(false);
  });
});
