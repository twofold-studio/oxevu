import { API_BASE } from "./client";

const ARTWORK_PREFIX = `${API_BASE}/items/`;

/**
 * Artwork URLs come from the server as relative same-origin paths. Anything
 * else (absolute, protocol-relative, outside the item artwork route) is
 * ignored so a malformed record can never make the browser fetch from a
 * third party or send the session cookie somewhere unexpected.
 */
export function artworkSrc(url: string | null | undefined): string | null {
  if (url && /^\/api\/v1\/acquisition\/artwork\/[a-zA-Z0-9]+\.jpg$/.test(url)) return url;
  if (!url || !url.startsWith(ARTWORK_PREFIX) || !url.includes("/artwork/")) return null;
  return url;
}
