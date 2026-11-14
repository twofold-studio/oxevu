import type { UserInput, UserSettings } from "$lib/api/admin-types";
import { MIN_PASSWORD_BYTES, type SyncplayAccess } from "$lib/api/types";

export interface UserDraft {
  username: string;
  password: string;
  is_admin: boolean;
  disabled: boolean;
  can_play: boolean;
  can_transcode: boolean;
  syncplay_access: SyncplayAccess;
  all_libraries: boolean;
  library_ids: string[];
}

export interface UserErrors {
  username: string | null;
  password: string | null;
}

/** New accounts start as ordinary viewers with no library access. */
export const newUserDraft = (): UserDraft => ({
  username: "",
  password: "",
  is_admin: false,
  disabled: false,
  can_play: true,
  can_transcode: true,
  syncplay_access: "join",
  all_libraries: false,
  library_ids: [],
});

export function draftFromUser(user: UserSettings): UserDraft {
  return {
    username: user.username,
    password: "",
    is_admin: user.is_admin,
    disabled: user.disabled,
    can_play: user.can_play,
    can_transcode: user.can_transcode,
    syncplay_access: user.syncplay_access,
    all_libraries: user.all_libraries,
    library_ids: [...user.library_ids],
  };
}

export const passwordBytes = (value: string) => new TextEncoder().encode(value).length;

export function validatePassword(value: string, required: boolean): string | null {
  if (!value) return required ? "Enter a password." : null;
  return passwordBytes(value) < MIN_PASSWORD_BYTES
    ? `Use at least ${MIN_PASSWORD_BYTES} characters.`
    : null;
}

export function validateUser(draft: UserDraft, creating: boolean): UserErrors {
  const username = draft.username.trim();
  return {
    username: !username
      ? "Enter a username."
      : username.length > 64
        ? "Use at most 64 characters."
        : /\s/.test(username)
          ? "Usernames can't contain spaces."
          : null,
    password: validatePassword(draft.password, creating),
  };
}

export const hasUserErrors = (errors: UserErrors) => Boolean(errors.username || errors.password);

/**
 * Request body. An empty password on update is omitted so the server keeps
 * the current one; grants only matter when not all libraries are allowed.
 */
export function toUserInput(draft: UserDraft, knownLibraryIds: string[]): UserInput {
  const input: UserInput = {
    username: draft.username.trim(),
    is_admin: draft.is_admin,
    disabled: draft.disabled,
    can_play: draft.can_play,
    can_transcode: draft.can_transcode,
    syncplay_access: draft.syncplay_access,
    all_libraries: draft.all_libraries,
    library_ids: draft.all_libraries
      ? []
      : draft.library_ids.filter((id) => knownLibraryIds.includes(id)),
  };
  if (draft.password) input.password = draft.password;
  return input;
}
