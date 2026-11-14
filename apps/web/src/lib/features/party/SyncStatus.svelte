<script lang="ts">
  import WifiOffIcon from "@lucide/svelte/icons/wifi-off";
  import type { SyncState } from "$lib/party/driver";
  import type { ConnectionState } from "$lib/party/connection.svelte";

  // Honest sync indicator: shows drift instead of claiming perfect sync.
  let { sync, connection }: { sync: SyncState; connection: ConnectionState } = $props();

  const label = $derived.by(() => {
    if (connection === "reconnecting") return "Reconnecting";
    switch (sync.kind) {
      case "loading":
        return "Syncing";
      case "blocked":
        return "Waiting for you";
      case "correcting":
        return `Catching up (${sync.drift > 0 ? "+" : "−"}${Math.abs(sync.drift).toFixed(1)} s)`;
      case "in-sync":
        return "In sync";
    }
  });
  const dot = $derived(
    connection === "reconnecting" || sync.kind === "blocked"
      ? "bg-destructive"
      : sync.kind === "in-sync"
        ? "bg-emerald-400"
        : "bg-amber-400",
  );
</script>

<span class="flex shrink-0 items-center gap-1.5 pt-1.5 text-xs text-white/80" role="status" aria-live="polite">
  {#if connection === "reconnecting"}
    <WifiOffIcon class="size-3.5" />
  {:else}
    <span class={`size-2 rounded-full ${dot}`} aria-hidden="true"></span>
  {/if}
  {label}
</span>
