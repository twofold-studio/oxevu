import { errorMessage } from "$lib/api/client";
import type { ReleaseResults } from "$lib/api/acquisition-types";
import { dedupeById, type SearchState, type SearchComplete } from "./release-search";

export class ReleaseSearch {
  state = $state<SearchState>({ kind: "idle" });
  private controller: AbortController | null = null;
  private previous: SearchState = { kind: "idle" };
  private latest: SearchComplete | null = null;

  constructor(
    private readonly fetch: (profile: string, signal: AbortSignal) => Promise<ReleaseResults>,
  ) {}

  async search(profile: string, total: number) {
    this.cancel();
    const controller = new AbortController();
    this.controller = controller;
    this.previous = this.state;
    const began = Date.now();
    this.state = { kind: "searching", startedAt: began, total, previous: this.latest };
    const deadline = setTimeout(() => controller.abort("timeout"), 90000);
    try {
      const raw = await this.fetch(profile, controller.signal);
      if (this.controller !== controller) return;
      const result = { ...raw, releases: dedupeById(raw.releases) };
      if (
        result.searched.indexers > 0 &&
        result.errors.length === result.searched.indexers &&
        result.releases.length === 0
      ) {
        this.state = {
          kind: "failed",
          message: "No indexer completed the search. Review the source errors below.",
          retryable: result.errors.some((error) => error.retryable),
          elapsed: Date.now() - began,
          result,
        };
      } else {
        const completed: SearchComplete = {
          kind: result.releases.length ? "results" : "empty",
          result,
          profile,
        };
        this.latest = completed;
        this.state = completed;
      }
    } catch (error) {
      if (this.controller !== controller) return;
      this.state = {
        kind: "failed",
        message:
          controller.signal.reason === "timeout"
            ? "The search did not finish. Some indexers did not respond."
            : errorMessage(error),
        retryable: true,
        elapsed: Date.now() - began,
      };
    } finally {
      clearTimeout(deadline);
      if (this.controller === controller) this.controller = null;
    }
  }

  cancel() {
    if (!this.controller) return;
    this.controller.abort("cancel");
    this.controller = null;
    this.state = this.previous;
  }
}
