// Thin fetch wrapper for /api/v1. The session is an HTTP-only cookie, so the
// browser attaches it to same-origin requests; nothing here touches tokens.

export const API_BASE = "/api/v1";

export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }

  get unauthorized() {
    return this.status === 401;
  }

  get forbidden() {
    return this.status === 403;
  }
}

type UnauthorizedListener = () => void;
let onUnauthorized: UnauthorizedListener | null = null;

/** Called whenever an authenticated request comes back 401 (expired session). */
export function setUnauthorizedListener(listener: UnauthorizedListener | null) {
  onUnauthorized = listener;
}

interface RequestOptions {
  method?: "GET" | "POST" | "PUT";
  body?: unknown;
  /** Survive page unload; used for the final progress write. */
  keepalive?: boolean;
  /** Auth endpoints report bad credentials as 401 without ending a session. */
  ignoreUnauthorized?: boolean;
  headers?: Record<string, string>;
  signal?: AbortSignal;
}

export async function request<T>(path: string, options: RequestOptions = {}): Promise<T> {
  const { method = "GET", body, keepalive, ignoreUnauthorized, headers: extra } = options;
  const headers: Record<string, string> = { Accept: "application/json", ...extra };
  if (body !== undefined) headers["Content-Type"] = "application/json";
  let response: Response;
  try {
    response = await fetch(`${API_BASE}${path}`, {
      method,
      credentials: "same-origin",
      keepalive,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      signal: options.signal,
    });
  } catch (error) {
    if (options.signal?.aborted) throw error;
    throw new ApiError(0, "Oxevu server is unreachable. Check that it is running.");
  }

  if (!response.ok) {
    const message = await readError(response);
    if (response.status === 401 && !ignoreUnauthorized) onUnauthorized?.();
    throw new ApiError(response.status, message);
  }
  if (response.status === 204) return undefined as T;
  return (await response.json()) as T;
}

async function readError(response: Response): Promise<string> {
  try {
    const data: unknown = await response.json();
    if (data && typeof data === "object" && "error" in data && typeof data.error === "string") {
      return data.error;
    }
  } catch {
    // Non-JSON error body (proxy page, empty response); fall through.
  }
  if (response.status === 401) return "Your session has ended. Sign in again.";
  if (response.status === 403) return "Your account can't do that.";
  if (response.status >= 500)
    return `The server failed to handle the request (${response.status}).`;
  return `Request failed (${response.status}).`;
}

export function errorMessage(error: unknown): string {
  if (error instanceof ApiError) return error.message;
  if (error instanceof Error) return error.message;
  return "Something went wrong.";
}
