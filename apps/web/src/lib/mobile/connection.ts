import { Capacitor, CapacitorHttp, registerPlugin } from "@capacitor/core";

interface ServerPlugin {
  getServer(): Promise<{ url: string | null }>;
  setServer(options: { url: string }): Promise<void>;
}

const settings = registerPlugin<ServerPlugin>("OxevuServer");
export const isMobileApp = Capacitor.isNativePlatform();
let server: string | null = null;

export function requestServerPicker() {
  window.dispatchEvent(new Event("oxevu:choose-server"));
}

export function serverOrigin(input: string): string {
  const url = new URL(input.trim());
  if (
    !["http:", "https:"].includes(url.protocol) ||
    url.username ||
    url.password ||
    url.search ||
    url.hash ||
    url.pathname !== "/"
  ) {
    throw new Error("Enter an HTTP or HTTPS server address without a path or password.");
  }
  return url.origin;
}

export async function loadMobileServer(): Promise<string | null> {
  if (!isMobileApp) return null;
  server = (await settings.getServer()).url;
  return server;
}

export async function connectMobileServer(input: string) {
  const url = serverOrigin(input);
  const result = await CapacitorHttp.get({
    url: `${url}/api/v1/auth/status`,
    connectTimeout: 10000,
    readTimeout: 10000,
    disableRedirects: true,
  });
  if (result.status !== 200 || typeof result.data?.setup_required !== "boolean") {
    throw new Error(
      "That address did not return an Oxevu server. Check the address and connection.",
    );
  }
  await settings.setServer({ url });
  globalThis.location.reload();
}

/** JSON SDK calls use the native cookie jar; credentials never enter web storage. */
export async function mobileFetch(input: RequestInfo | URL, init?: RequestInit): Promise<Response> {
  if (!server) throw new Error("Choose an Oxevu server first.");
  const request = new Request(input, init);
  if (request.signal.aborted) throw new DOMException("Aborted", "AbortError");
  const path = new URL(request.url).pathname;
  if (!path.startsWith("/api/v1/")) throw new Error("Invalid API path");
  const headers = Object.fromEntries(request.headers);
  // This native request originates from the server selected by the app's user.
  headers.Origin = server;
  const body = ["GET", "HEAD"].includes(request.method) ? undefined : await request.text();
  const response = await CapacitorHttp.request({
    url: server + path + new URL(request.url).search,
    method: request.method,
    headers,
    data: body ? JSON.parse(body) : undefined,
    connectTimeout: 10000,
    readTimeout: 100000,
    disableRedirects: true,
    responseType: "json",
  });
  if (request.signal.aborted) throw new DOMException("Aborted", "AbortError");
  return new Response(response.status === 204 ? null : JSON.stringify(response.data), {
    status: response.status,
    headers: { "Content-Type": "application/json" },
  });
}
