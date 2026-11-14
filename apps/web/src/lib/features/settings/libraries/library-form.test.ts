import { describe, expect, test } from "vite-plus/test";
import {
  contains,
  draftFrom,
  hasErrors,
  normalizePath,
  toInput,
  validateLibrary,
} from "./library-form";

const draft = (paths: string[], name = "Movies") => ({
  name,
  kind: "movies" as const,
  paths,
  enabled: true,
});

describe("library form", () => {
  test("accepts several separate absolute roots", () => {
    const errors = validateLibrary(draft(["/srv/a/movies", "/mnt/b/movies/"]));
    expect(hasErrors(errors)).toBe(false);
    expect(toInput(draft(["/srv/a/movies", " /mnt/b/movies/ ", ""])).paths).toEqual([
      "/srv/a/movies",
      "/mnt/b/movies",
    ]);
  });

  test("requires a name and at least one folder", () => {
    const errors = validateLibrary(draft([""], " "));
    expect(errors.name).toBe("Enter a name.");
    expect(errors.paths).toBe("Add at least one folder.");
  });

  test("rejects relative, duplicate and nested roots", () => {
    const errors = validateLibrary(draft(["media", "/srv/a", "/srv/a/", "/srv/a/sub"]));
    expect(errors.path[0]).toMatch(/absolute/);
    expect(errors.path[1]).toBeNull();
    expect(errors.path[2]).toMatch(/already listed/);
    expect(errors.path[3]).toMatch(/Already included by \/srv\/a/);
  });

  test("blank extra rows are flagged once a folder is given", () => {
    expect(validateLibrary(draft(["/srv/a", ""])).path[1]).toMatch(/remove this row/);
  });

  test("containment matches whole path segments", () => {
    expect(contains("/srv/a", "/srv/a/b")).toBe(true);
    expect(contains("/srv/a", "/srv/ab")).toBe(false);
    expect(normalizePath("/")).toBe("/");
  });

  test("older records without paths edit their single root", () => {
    const legacy = { id: "1", name: "M", kind: "movies" as const, path: "/m" } as never;
    expect(draftFrom(legacy)).toEqual({ name: "M", kind: "movies", paths: ["/m"], enabled: true });
  });
});
