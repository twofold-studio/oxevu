<script lang="ts">
  import { onMount } from "svelte";
  import { toast } from "svelte-sonner";
  import UsersRoundIcon from "@lucide/svelte/icons/users-round";
  import PlayIcon from "@lucide/svelte/icons/play";
  import { Button } from "$lib/components/ui/button";
  import type { Item } from "$lib/api/types";
  import type { PartyControl } from "$lib/api/playback-types";
  import { errorMessage } from "$lib/api/client";
  import { session } from "$lib/state/session.svelte";
  import { href } from "$lib/state/router.svelte";
  import { PlaybackEngine } from "$lib/playback/engine.svelte";
  import { SyncDriver, type SyncState } from "$lib/party/driver";
  import { expectedPosition } from "$lib/party/sync";
  import type { PartyConnection } from "$lib/party/connection.svelte";
  import PlayerView, { type PlayerIntent } from "$lib/features/player/PlayerView.svelte";
  import PartyPanel from "./PartyPanel.svelte";
  import SyncStatus from "./SyncStatus.svelte";

  // One party title. Keyed by item in PartyRoom, so a host's title change
  // tears this down (stopping its stream) and starts a fresh engine.
  let {
    item,
    connection,
    isHost,
    onleave,
  }: { item: Item; connection: PartyConnection; isHost: boolean; onleave: () => void } = $props();

  const canTranscode = session.user?.can_transcode ?? false;
  // svelte-ignore state_referenced_locally
  const engine = new PlaybackEngine(item, canTranscode, () => void session.load());
  const driver = new SyncDriver(engine);
  // svelte-ignore state_referenced_locally
  const start = connection.party ? expectedPosition(connection.party, connection.serverNow(), item.duration_seconds) : 0;

  // svelte-ignore state_referenced_locally
  const autoplay = !(connection.party?.paused ?? true);

  let sync = $state<SyncState>({ kind: "loading" });
  let panelOpen = $state(false);

  function tick() {
    const party = connection.party;
    if (!party || party.item_id !== item.id) return;
    sync = driver.apply(party, connection.serverNow(), Date.now(), engine.duration);
  }

  onMount(() => {
    // Polls arrive every second; correcting twice as often keeps
    // extrapolation smooth between them.
    const timer = setInterval(tick, 500);
    return () => clearInterval(timer);
  });

  async function send(body: PartyControl) {
    try {
      await connection.control(body);
    } catch (error) {
      // The next snapshot puts the host back on the server timeline.
      toast.error(`The party didn't accept that: ${errorMessage(error)}`);
    }
  }

  function onintent(intent: PlayerIntent) {
    if (!isHost) return;
    driver.noteLocalIntent(Date.now());
    if (intent.type === "seek") void send({ action: "seek", position_seconds: intent.position });
    else void send({ action: intent.type, position_seconds: engine.absolute });
  }

  async function changeItem(itemId: string) {
    try {
      await connection.control({ action: "item", item_id: itemId, position_seconds: 0 });
    } catch (error) {
      toast.error(`Couldn't change the title: ${errorMessage(error)}`);
      throw error;
    }
  }
</script>

<PlayerView
  {engine}
  {start}
  backHref={href.parties()}
  {canTranscode}
  {autoplay}
  locked={!isHost}
  {onintent}
>
  {#snippet status()}
    <SyncStatus {sync} connection={connection.state} />
    <Button
      variant="ghost"
      size="icon-lg"
      onclick={() => (panelOpen = !panelOpen)}
      aria-label="Watch party"
      aria-expanded={panelOpen}
    >
      <UsersRoundIcon />
    </Button>
  {/snippet}
  {#snippet aside()}
    {#if sync.kind === "blocked" && !isHost}
      <!-- Autoplay was blocked: a click here is the user gesture browsers need. -->
      <div class="absolute inset-0 z-10 grid place-items-center bg-black/60">
        <div class="flex flex-col items-center gap-3 text-center">
          <p class="text-sm text-white/80">The party is playing.</p>
          <Button size="lg" onclick={() => void engine.play()}>
            <PlayIcon data-icon="inline-start" />
            Join playback
          </Button>
        </div>
      </div>
    {/if}
    {#if panelOpen && connection.party}
      <PartyPanel
        party={connection.party}
        {isHost}
        onclose={() => (panelOpen = false)}
        {onleave}
        onchangeitem={changeItem}
      />
    {/if}
  {/snippet}
</PlayerView>
