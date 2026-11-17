<script lang="ts">
  import { untrack } from "svelte";
  import { Button } from "$lib/components/ui/button";
  import { Input } from "$lib/components/ui/input";
  import { connectMobileServer } from "./connection";

  let { current = null, oncancel }: { current?: string | null; oncancel?: () => void } = $props();
  let address = $state(untrack(() => current ?? ""));
  let busy = $state(false);
  let error = $state<string | null>(null);

  async function connect(event: SubmitEvent) {
    event.preventDefault();
    busy = true;
    error = null;
    try { await connectMobileServer(address); }
    catch (reason) {
      error = reason instanceof Error ? reason.message : "Could not connect to the server.";
      busy = false;
    }
  }
</script>

<main class="grid min-h-dvh place-items-center p-6">
  <form onsubmit={connect} class="w-full max-w-sm space-y-6 rounded-xl border bg-card p-6">
    <div class="space-y-2">
      <h1 class="text-2xl font-semibold tracking-tight">Connect to Oxevu</h1>
      <p class="text-sm text-muted-foreground">Enter your server address. Your phone must be on the same network or connected through NetBird.</p>
    </div>
    <div class="space-y-2">
      <label for="server" class="text-sm font-medium">Server address</label>
      <Input id="server" type="url" bind:value={address} placeholder="http://vd.netbird.cloud:8096" required disabled={busy} autocomplete="url" />
      <p class="text-xs text-muted-foreground">Use HTTPS when connecting over the internet.</p>
    </div>
    {#if error}<p role="alert" class="text-sm text-destructive">{error}</p>{/if}
    <Button type="submit" class="w-full" disabled={busy}>{busy ? "Connecting…" : "Connect"}</Button>
    {#if current && oncancel}<Button variant="outline" class="w-full" type="button" onclick={oncancel} disabled={busy}>Cancel</Button>{/if}
  </form>
</main>
