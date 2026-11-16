import type { Component } from "svelte";
import FolderOpenIcon from "@lucide/svelte/icons/folder-open";
import DatabaseIcon from "@lucide/svelte/icons/database";
import CaptionsIcon from "@lucide/svelte/icons/captions";
import CpuIcon from "@lucide/svelte/icons/cpu";
import GaugeIcon from "@lucide/svelte/icons/gauge";
import HardDriveIcon from "@lucide/svelte/icons/hard-drive";
import HistoryIcon from "@lucide/svelte/icons/history";
import DownloadIcon from "@lucide/svelte/icons/download";
import NetworkIcon from "@lucide/svelte/icons/network";
import ListOrderedIcon from "@lucide/svelte/icons/list-ordered";
import ArrowUpFromLineIcon from "@lucide/svelte/icons/arrow-up-from-line";
import FolderInputIcon from "@lucide/svelte/icons/folder-input";
import RadioTowerIcon from "@lucide/svelte/icons/radio-tower";
import SlidersHorizontalIcon from "@lucide/svelte/icons/sliders-horizontal";
import ImportIcon from "@lucide/svelte/icons/import";
import UsersIcon from "@lucide/svelte/icons/users";
import ShieldCheckIcon from "@lucide/svelte/icons/shield-check";
import UserRoundIcon from "@lucide/svelte/icons/user-round";
import KeyRoundIcon from "@lucide/svelte/icons/key-round";
import MonitorPlayIcon from "@lucide/svelte/icons/monitor-play";
import type { SettingsPageId } from "$lib/state/settings-pages";

export const pageIcons: Record<SettingsPageId, Component> = {
  "media/libraries": FolderOpenIcon,
  "media/metadata": DatabaseIcon,
  "media/subtitles": CaptionsIcon,
  "playback/conversion": CpuIcon,
  "playback/quality": GaugeIcon,
  "playback/scratch": HardDriveIcon,
  "playback/resume": HistoryIcon,
  "downloads/storage": DownloadIcon,
  "downloads/network": NetworkIcon,
  "downloads/queue": ListOrderedIcon,
  "downloads/seeding": ArrowUpFromLineIcon,
  "downloads/imports": FolderInputIcon,
  "sources/indexers": RadioTowerIcon,
  "sources/profiles": SlidersHorizontalIcon,
  "sources/migration": ImportIcon,
  "people/users": UsersIcon,
  "people/requests": ShieldCheckIcon,
  "account/profile": UserRoundIcon,
  "account/password": KeyRoundIcon,
  "account/player": MonitorPlayIcon,
};
