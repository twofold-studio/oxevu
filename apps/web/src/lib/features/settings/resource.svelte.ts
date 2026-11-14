import { ApiError, errorMessage } from "$lib/api/client";

export type ResourceStatus = "loading" | "ready" | "forbidden" | "error";

/**
 * Load state for an admin resource. A 403 is its own state so pages explain
 * missing permission instead of showing a generic failure; 401 is handled by
 * the client (the session ends).
 */
export function createResource<T>(fetcher: () => Promise<T>) {
  let value = $state<T | null>(null);
  let status = $state<ResourceStatus>("loading");
  let error = $state<string | null>(null);
  let token = 0;

  async function load() {
    const current = ++token;
    if (value === null) status = "loading";
    error = null;
    try {
      const result = await fetcher();
      if (current !== token) return;
      value = result;
      status = "ready";
    } catch (e) {
      if (current !== token) return;
      if (e instanceof ApiError && e.forbidden) status = "forbidden";
      else {
        error = errorMessage(e);
        status = "error";
      }
    }
  }

  return {
    get value() {
      return value;
    },
    set value(next: T | null) {
      value = next;
    },
    get status() {
      return status;
    },
    get error() {
      return error;
    },
    load,
  };
}
