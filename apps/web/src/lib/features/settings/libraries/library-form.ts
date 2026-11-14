import type { Library, LibraryInput, LibraryKind } from "$lib/api/types";

// Client-side checks give quick feedback only. The server canonicalizes
// roots (symlinks, mounts) and rejects overlaps with other libraries and
// with its own data directories; its message is shown when it refuses.

export interface LibraryDraft {
  name: string;
  kind: LibraryKind;
  paths: string[];
  enabled: boolean;
}

export interface LibraryErrors {
  name: string | null;
  /** Problem with the list as a whole (none given). */
  paths: string | null;
  /** One entry per draft path. */
  path: (string | null)[];
}

export const emptyDraft = (): LibraryDraft => ({
  name: "",
  kind: "movies",
  paths: [""],
  enabled: true,
});

export function draftFrom(library: Library): LibraryDraft {
  const paths = library.paths?.length ? library.paths : [library.path];
  return {
    name: library.name,
    kind: library.kind,
    paths: [...paths],
    enabled: library.enabled ?? true,
  };
}

/** Trim and drop trailing separators (but keep a bare root). */
export function normalizePath(path: string): string {
  const trimmed = path.trim();
  const stripped = trimmed.replace(/[\\/]+$/, "");
  return stripped || trimmed.slice(0, 1);
}

const isAbsolute = (path: string) => path.startsWith("/") || /^[A-Za-z]:[\\/]/.test(path);

/** True when `child` is `parent` or lies inside it. */
export function contains(parent: string, child: string): boolean {
  if (parent === child) return true;
  const base = parent.endsWith("/") || parent.endsWith("\\") ? parent : `${parent}/`;
  return child.startsWith(base) || child.startsWith(base.replace(/\/$/, "\\"));
}

export function validateLibrary(draft: LibraryDraft): LibraryErrors {
  const name = draft.name.trim();
  const normalized = draft.paths.map(normalizePath);
  const given = normalized.filter(Boolean);

  const path = normalized.map((value, index) => {
    if (!value) return given.length > 0 ? "Enter a folder or remove this row." : null;
    if (!isAbsolute(value)) return "Use an absolute path, such as /srv/media/movies.";
    const earlier = normalized.slice(0, index);
    if (earlier.includes(value)) return "This folder is already listed.";
    const other = normalized.find((p, i) => i !== index && p && p !== value && contains(p, value));
    if (other) return `Already included by ${other}.`;
    return null;
  });

  return {
    name: !name ? "Enter a name." : name.length > 100 ? "Use at most 100 characters." : null,
    paths: given.length === 0 ? "Add at least one folder." : null,
    path,
  };
}

export const hasErrors = (errors: LibraryErrors) =>
  Boolean(errors.name || errors.paths || errors.path.some(Boolean));

export function toInput(draft: LibraryDraft): LibraryInput {
  return {
    name: draft.name.trim(),
    kind: draft.kind,
    paths: draft.paths.map(normalizePath).filter(Boolean),
    enabled: draft.enabled,
  };
}
