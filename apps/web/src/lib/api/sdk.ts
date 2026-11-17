import { createClient } from "./generated/client";
import { API_BASE, ApiError, notifyUnauthorized, readError } from "./client";
import { isMobileApp, mobileFetch } from "$lib/mobile/connection";

/** Keep session/error policy outside generated files so regeneration is safe. */
export function sdkClient(ignoreUnauthorized = false) {
  return createClient({
    baseUrl: new URL(API_BASE, globalThis.location?.origin ?? "http://localhost").href,
    credentials: "same-origin",
    fetch: async (input, init) => {
      let response: Response;
      try {
        response = await (isMobileApp ? mobileFetch(input, init) : fetch(input, init));
      } catch (error) {
        const signal = input instanceof Request ? input.signal : init?.signal;
        if (signal?.aborted) throw error;
        throw new ApiError(0, "Oxevu server is unreachable. Check that it is running.");
      }
      if (!response.ok) {
        if (response.status === 401 && !ignoreUnauthorized) notifyUnauthorized();
        throw new ApiError(response.status, await readError(response));
      }
      return response;
    },
  });
}

export async function sdkData<T>(operation: Promise<{ data?: T }>): Promise<T> {
  const result = await operation;
  if (result.data === undefined) throw new ApiError(0, "The server returned an empty response.");
  return result.data;
}
