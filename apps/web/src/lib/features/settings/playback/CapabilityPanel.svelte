<script lang="ts">
  import CircleCheckIcon from "@lucide/svelte/icons/circle-check";
  import CircleXIcon from "@lucide/svelte/icons/circle-x";
  import type { Capabilities } from "$lib/api/admin-types";

  // What the server actually probed. Paths come from its environment
  // (OXEVU_FFMPEG, OXEVU_TRANSCODE_DIR) and are read-only here.
  let { capabilities }: { capabilities: Capabilities } = $props();

  const checks = $derived([
    { label: "FFmpeg", ok: capabilities.ffmpeg_available },
    { label: "NVIDIA encoder (NVENC)", ok: capabilities.nvenc_available },
    { label: "Hardware decoding", ok: capabilities.hardware_decode_available },
  ]);
</script>

<div class="flex flex-col gap-4">
  <ul class="divide-y rounded-lg border text-sm">
    {#each checks as check (check.label)}
      <li class="flex items-center justify-between gap-4 px-4 py-2.5">
        <span>{check.label}</span>
        <span class="flex items-center gap-1.5 text-muted-foreground">
          {#if check.ok}
            <CircleCheckIcon class="size-4 text-foreground" />Available
          {:else}
            <CircleXIcon class="size-4" />Unavailable
          {/if}
        </span>
      </li>
    {/each}
  </ul>

  <dl class="grid gap-3 text-sm">
    <div class="flex flex-col gap-1">
      <dt class="text-muted-foreground">FFmpeg executable · <code class="font-mono text-xs">OXEVU_FFMPEG</code></dt>
      <dd class="font-mono text-xs break-all">{capabilities.ffmpeg_path ?? "Not found"}</dd>
    </div>
    <div class="flex flex-col gap-1">
      <dt class="text-muted-foreground">
        Scratch folder · <code class="font-mono text-xs">OXEVU_TRANSCODE_DIR</code>
      </dt>
      <dd class="font-mono text-xs break-all">{capabilities.scratch_path ?? "Not configured"}</dd>
    </div>
  </dl>
  <p class="text-xs leading-5 text-muted-foreground">
    Set these in the server's environment and restart it. The scratch folder must be outside every library folder.
  </p>

  {#if capabilities.notes.length > 0}
    <ul class="flex flex-col gap-1.5 rounded-lg border bg-muted/30 p-3 text-xs leading-5 text-muted-foreground">
      {#each capabilities.notes as note, index (index)}
        <li>{note}</li>
      {/each}
    </ul>
  {/if}
</div>
