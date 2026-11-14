// Administration resources from docs/api-playback-settings.md.
import type { SyncplayAccess, UserAccess } from "./types";

/** A user as administrators see and edit it. */
export interface UserSettings extends UserAccess {
  id: string;
  username: string;
  all_libraries: boolean;
  library_ids: string[];
}

export interface UserInput extends UserAccess {
  username: string;
  /** Required on create; omit on update to keep the current password. */
  password?: string;
  all_libraries: boolean;
  library_ids: string[];
}

export type { SyncplayAccess };

export type HardwareAcceleration = "none" | "nvenc";

export interface Settings {
  transcoding_enabled: boolean;
  hardware_acceleration: HardwareAcceleration;
  hardware_encoding: boolean;
  hardware_decoding: boolean;
  gpu_index: number;
  allow_cpu_fallback: boolean;
  max_concurrent_transcodes: number;
  max_video_bitrate_mbps: number;
  max_height: number;
  cpu_crf: number;
  scratch_limit_gb: number;
  session_idle_seconds: number;
  resume_min_percent: number;
  resume_max_percent: number;
  resume_min_duration_seconds: number;
}

/** Runtime probe results; paths come from server environment, not settings. */
export interface Capabilities {
  ffmpeg_available: boolean;
  nvenc_available: boolean;
  hardware_decode_available: boolean;
  ffmpeg_path: string | null;
  scratch_path: string | null;
  notes: string[];
}

export interface SettingsEnvelope {
  settings: Settings;
  capabilities: Capabilities;
}

export interface PasswordChange {
  current_password: string;
  new_password: string;
}
