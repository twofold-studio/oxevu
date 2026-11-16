import { describe, expect, test } from "vite-plus/test";
import type { AcquisitionPermissions } from "$lib/api/acquisition-types";
import {
  canOpenSettingsPage,
  resolveSettings,
  settingsAliases,
  settingsNavFor,
  settingsPageIds,
  type SettingsResolution,
  type SettingsTarget,
} from "./settings-pages";

const permissions = (overrides: Partial<AcquisitionPermissions> = {}): AcquisitionPermissions => ({
  can_request: true,
  auto_approve: false,
  manage_requests: false,
  manage_downloads: false,
  manage_indexers: false,
  request_limit: 10,
  ...overrides,
});
const admin = { is_admin: true };
const viewer = { is_admin: false, acquisition: permissions() };
// Request reviewers and transfer managers have no settings beyond their account.
const reviewer = {
  is_admin: false,
  acquisition: permissions({ manage_requests: true, manage_downloads: true }),
};
const indexerManager = { is_admin: false, acquisition: permissions({ manage_indexers: true }) };
type User = typeof admin | typeof viewer | null;

const navIds = (user: User) =>
  settingsNavFor(user).flatMap((group) => group.pages.map((page) => page.id));

/** Table rows: who opens which address, and what should happen. */
const cases = (rows: [User, SettingsTarget, SettingsResolution][]) => {
  for (const [user, target, expected] of rows)
    expect(resolveSettings(target, user)).toEqual(expected);
};

describe("settings navigation by role", () => {
  test("administrators see every page, in catalog order", () => {
    expect(navIds(admin)).toEqual(settingsPageIds);
  });

  test("viewers see only their own account", () => {
    for (const user of [viewer, reviewer, null]) {
      expect(navIds(user)).toEqual(["account/profile", "account/password", "account/player"]);
      expect(settingsNavFor(user).map((group) => group.id)).toEqual(["account"]);
    }
  });

  test("delegated indexer managers see indexers and their account", () => {
    expect(navIds(indexerManager)).toEqual([
      "sources/indexers",
      "account/profile",
      "account/password",
      "account/player",
    ]);
  });

  test("navigation and direct access agree", () => {
    for (const user of [admin, viewer, reviewer, indexerManager, null])
      for (const page of settingsPageIds)
        expect(navIds(user).includes(page)).toBe(canOpenSettingsPage(user, page));
  });
});

describe("settings URL resolution", () => {
  test("the bare index picks the first permitted page", () => {
    cases([
      [admin, null, { kind: "index", first: "media/libraries" }],
      [viewer, null, { kind: "index", first: "account/profile" }],
      [indexerManager, null, { kind: "index", first: "sources/indexers" }],
    ]);
  });

  test("old tab links redirect to their replacement pages", () => {
    cases([
      [admin, "libraries", { kind: "redirect", page: "media/libraries" }],
      [admin, "playback", { kind: "redirect", page: "playback/conversion" }],
      [admin, "integrations", { kind: "redirect", page: "media/metadata" }],
      [admin, "acquisition", { kind: "redirect", page: "downloads/storage" }],
      [admin, "users", { kind: "redirect", page: "people/users" }],
      [admin, "account", { kind: "redirect", page: "account/profile" }],
      [viewer, "account", { kind: "redirect", page: "account/profile" }],
    ]);
  });

  test("the old acquisition tab takes indexer managers to indexers", () => {
    cases([
      [indexerManager, "acquisition", { kind: "redirect", page: "sources/indexers" }],
      [indexerManager, "sources", { kind: "redirect", page: "sources/indexers" }],
    ]);
  });

  test("forbidden pages and aliases explain instead of redirecting", () => {
    cases([
      [viewer, "playback/conversion", { kind: "forbidden", page: "playback/conversion" }],
      [viewer, "libraries", { kind: "forbidden", page: "media/libraries" }],
      [viewer, "acquisition", { kind: "forbidden", page: "downloads/storage" }],
      [reviewer, "sources/indexers", { kind: "forbidden", page: "sources/indexers" }],
      [indexerManager, "sources/profiles", { kind: "forbidden", page: "sources/profiles" }],
      [indexerManager, "downloads/storage", { kind: "forbidden", page: "downloads/storage" }],
      [indexerManager, "people/requests", { kind: "forbidden", page: "people/requests" }],
    ]);
  });

  test("permitted pages open directly", () => {
    cases([
      [indexerManager, "sources/indexers", { kind: "page", page: "sources/indexers" }],
      [viewer, "account/password", { kind: "page", page: "account/password" }],
    ]);
  });

  test("every alias covers at least one page", () => {
    for (const pages of Object.values(settingsAliases)) expect(pages.length).toBeGreaterThan(0);
  });
});
