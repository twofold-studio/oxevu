import { describe, expect, test } from "vite-plus/test";
import { newUserDraft, toUserInput, validateUser } from "./user-form";

describe("user form", () => {
  test("new users need a username and a long enough password", () => {
    const errors = validateUser({ ...newUserDraft(), username: " ", password: "short" }, true);
    expect(errors.username).toBe("Enter a username.");
    expect(errors.password).toMatch(/at least 12/);
    expect(validateUser({ ...newUserDraft(), username: "sam" }, true).password).toBe(
      "Enter a password.",
    );
  });

  test("password counts UTF-8 bytes like the server", () => {
    // Four 3-byte characters = 12 bytes.
    expect(
      validateUser({ ...newUserDraft(), username: "sam", password: "日本語字" }, true).password,
    ).toBeNull();
  });

  test("editing keeps the password when left empty", () => {
    const draft = { ...newUserDraft(), username: "sam" };
    expect(validateUser(draft, false).password).toBeNull();
    expect("password" in toUserInput(draft, [])).toBe(false);
    expect(toUserInput({ ...draft, password: "a-new-long-password" }, []).password).toBe(
      "a-new-long-password",
    );
  });

  test("rejects usernames with spaces", () => {
    expect(
      validateUser({ ...newUserDraft(), username: "a b", password: "x".repeat(12) }, true).username,
    ).toMatch(/spaces/);
  });

  test("drops grants for unknown libraries and when all libraries are allowed", () => {
    const draft = { ...newUserDraft(), username: "sam", library_ids: ["a", "gone"] };
    expect(toUserInput(draft, ["a", "b"]).library_ids).toEqual(["a"]);
    expect(toUserInput({ ...draft, all_libraries: true }, ["a"]).library_ids).toEqual([]);
  });
});
