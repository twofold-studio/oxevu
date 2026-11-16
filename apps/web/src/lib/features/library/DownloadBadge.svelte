<script lang="ts">
  import ArrowDownIcon from "@lucide/svelte/icons/arrow-down";
  import PauseIcon from "@lucide/svelte/icons/pause";
  import ClockIcon from "@lucide/svelte/icons/clock";
  import CircleXIcon from "@lucide/svelte/icons/circle-x";
  import type { DownloadStatus } from "$lib/api/types";
  import { Badge } from "$lib/components/ui/badge";

  let { download, compact = false }: { download: DownloadStatus; compact?: boolean } = $props();

  const percent = $derived(
    download.size_bytes && download.size_bytes > 0
      ? Math.max(0, Math.min(100, Math.round((100 * download.bytes_received) / download.size_bytes)))
      : null,
  );

  const label = $derived(
    download.state === "downloading"
      ? percent !== null ? `${percent}%` : "Downloading"
      : download.state === "paused"
        ? "Paused"
        : download.state === "queued"
          ? "Queued"
          : download.state === "error"
            ? "Error"
            : download.state === "checking" ? "Checking"
            : ["seeding", "completed"].includes(download.state) ? "Downloaded"
            : download.state === "identity_changed" ? "Review title"
            : download.state === "removed" ? "Removed"
            : "Unavailable",
  );
</script>

<Badge variant="secondary"
  title={`Download: ${download.state}${percent !== null ? ` (${percent}%)` : ""}`}
>
  {#if download.state === "downloading"}
    <ArrowDownIcon data-icon="inline-start" />
  {:else if download.state === "paused"}
    <PauseIcon data-icon="inline-start" />
  {:else if download.state === "error"}
    <CircleXIcon data-icon="inline-start" />
  {:else}
    <ClockIcon data-icon="inline-start" />
  {/if}
  {label}
  {#if download.state === "downloading" && percent !== null && !compact}
    <span
      class="inline-block h-1 w-8 overflow-hidden rounded-full bg-muted"
      role="progressbar"
      aria-label="Download progress"
      aria-valuenow={percent}
      aria-valuemin={0}
      aria-valuemax={100}
    >
      <span class="block h-full bg-primary" style:width={`${percent}%`}></span>
    </span>
  {/if}
</Badge>
