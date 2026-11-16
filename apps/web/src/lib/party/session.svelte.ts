import { itemsApi } from "$lib/api/endpoints";
import { toast } from "svelte-sonner";
import { href, router } from "$lib/state/router.svelte";
import { resumePosition } from "$lib/media/progress";
import { PartyConnection } from "./connection.svelte";

/** One tab's membership survives navigation; only explicit Leave ends it. */
export class PartySession {
  connection = $state<PartyConnection | null>(null);
  private storageKey = "";
  private joining: Promise<void> | null = null;
  private generation = 0;

  restore(userId: string) {
    this.storageKey = `oxevu.party.${userId}`;
    const id = sessionStorage.getItem(this.storageKey);
    if (id) void this.join(id).catch(() => undefined);
  }

  async join(id: string) {
    if (this.connection?.id === id) return this.joining;
    this.leave();
    let seen: number | null = null;
    const connection = new PartyConnection(id, (party) => {
      if (connection !== this.connection) return;
      if (party.item_id && party.playback_revision !== seen) {
        this.generation++;
        router.go(href.watch(party.item_id));
      }
      seen = party.playback_revision;
    });
    this.connection = connection;
    if (this.storageKey) sessionStorage.setItem(this.storageKey, id);
    this.joining = connection.start().then(() => {
      if (connection.state !== "live")
        throw new Error(connection.message ?? "Couldn't join party.");
    });
    try {
      await this.joining;
    } finally {
      if (connection === this.connection) this.joining = null;
    }
  }

  async play(itemId: string, fromStart: boolean) {
    const generation = ++this.generation;
    const connection = this.connection;
    if (!connection) return;
    await this.joining;
    if (connection !== this.connection || generation !== this.generation) return;
    if (connection.state !== "live") throw new Error(connection.message ?? "Party disconnected.");
    if (connection.party?.item_id === itemId && !fromStart) return;
    const { item } = await itemsApi.get(itemId);
    if (connection !== this.connection || generation !== this.generation) return;
    await connection.control({
      action: "item",
      item_id: itemId,
      position_seconds: fromStart ? 0 : (resumePosition(item) ?? 0),
    });
  }

  leave() {
    this.generation++;
    this.connection?.leave();
    this.connection = null;
    if (this.storageKey) sessionStorage.removeItem(this.storageKey);
  }

  stop() {
    this.generation++;
    this.connection?.stop();
    this.connection = null;
  }

  checkClosed() {
    const connection = this.connection;
    if (connection?.state !== "closed" && connection?.state !== "failed") return;
    toast.error(connection.message ?? "Party ended.");
    router.go(href.parties());
    this.leave();
  }
}

export const partySession = new PartySession();
