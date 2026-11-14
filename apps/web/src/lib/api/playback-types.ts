// Playback sessions and watch parties from docs/api-playback-settings.md.

export type PlaybackMode = "auto" | "direct" | "transcode";
export type PlaybackMethod = "direct" | "remux" | "cpu" | "nvenc";

export interface PlaybackSessionRequest {
  item_id: string;
  start_seconds: number;
  mode: PlaybackMode;
  direct_play_supported: boolean;
}

export interface PlaybackSession {
  id: string;
  item_id: string;
  method: PlaybackMethod;
  status: "starting" | "ready" | "failed";
  stream_url: string | null;
  /** Absolute item time at which the stream begins (HLS media time 0). */
  start_seconds: number;
  duration_seconds: number | null;
  /** Why the server picked this method, e.g. a GPU fallback. */
  reason: string | null;
  error: string | null;
}

export interface PartyMember {
  user_id: string;
  username: string;
}

export interface Party {
  id: string;
  name: string;
  host_user_id: string;
  item_id: string;
  paused: boolean;
  /** Item position at `updated_at_ms` on the server clock. */
  position_seconds: number;
  updated_at_ms: number;
  server_now_ms: number;
  revision: number;
  members: PartyMember[];
}

export type PartyControl =
  | { action: "play"; position_seconds?: number }
  | { action: "pause"; position_seconds?: number }
  | { action: "seek"; position_seconds: number }
  | { action: "item"; item_id: string; position_seconds?: number };
