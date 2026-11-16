import type { Download, Release, ReleaseResults } from "$lib/api/acquisition-types";

export function dedupeById(releases: Release[]): Release[] {
  const unique = new Map<string, Release>();
  for (const release of releases) if (!unique.has(release.id)) unique.set(release.id, release);
  if (unique.size !== releases.length)
    console.warn("Removed duplicate release IDs from search results.");
  return [...unique.values()];
}
export function downloadFailure(download: Download): string | null {
  if (["error", "duplicate"].includes(download.state))
    return download.error ?? "This transfer could not be queued.";
  return null;
}
export type SearchComplete = { kind: "results" | "empty"; result: ReleaseResults; profile: string };
export type SearchState =
  | { kind: "idle" }
  | { kind: "searching"; startedAt: number; total: number; previous: SearchComplete | null }
  | SearchComplete
  | {
      kind: "failed";
      message: string;
      retryable: boolean;
      elapsed: number;
      result?: ReleaseResults;
    };
