// Shared session and error policy for generated REST clients.

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

export async function readError(response: Response): Promise<string> {
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

/** Shared session policy for generated clients. */
export function notifyUnauthorized() {
  onUnauthorized?.();
}

export function errorMessage(error: unknown): string {
  if (error instanceof ApiError) return error.message;
  if (error instanceof Error) return error.message;
  return "Something went wrong.";
}
