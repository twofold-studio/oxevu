import { itemsApi, partiesApi } from "$lib/api/endpoints";
import type { Item } from "$lib/api/types";
import { ApiError, errorMessage } from "$lib/api/client";
import type { Party, PartyControl } from "$lib/api/playback-types";
import { ClockEstimator } from "./clock";

export type ConnectionState = "joining" | "live" | "reconnecting" | "closed" | "failed";

const POLL_MS = 1000;
/** Consecutive failed polls before the room shows it is reconnecting. */
const RECONNECT_AFTER = 3;

/**
 * Membership in one watch party: joins, polls the snapshot every second,
 * estimates the server clock and sends host controls. Snapshots with an older
 * revision than one already seen are ignored (responses can overtake each
 * other); equal revisions still refresh membership.
 */
export class PartyConnection {
  party = $state<Party | null>(null);
  state = $state<ConnectionState>("joining");
  message = $state<string | null>(null);
  /**
   * The party's current title, fetched with this user's own authorization:
   * membership never grants library access.
   */
  item = $state<Item | null>(null);
  itemError = $state<string | null>(null);
  private loadingItem: string | null = null;

  private readonly clock = new ClockEstimator();
  private seen: number | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private failures = 0;
  private stopped = false;
  private left = false;
  readonly id: string;

  constructor(id: string) {
    this.id = id;
  }

  async start() {
    try {
      await this.timed(() => partiesApi.join(this.id));
      this.state = "live";
      this.schedule();
    } catch (error) {
      this.close(error, true);
    }
  }

  /** Server time now, in epoch milliseconds. */
  serverNow(): number {
    return this.clock.serverNow(Date.now());
  }

  /** Host only; the server rejects others and items any member can't open. */
  async control(body: PartyControl) {
    await this.timed(() => partiesApi.control(this.id, body));
  }

  /** Leave the party. For the host this ends it for everyone. */
  leave(keepalive = false) {
    this.stop();
    if (this.left) return;
    this.left = true;
    void partiesApi.leave(this.id, keepalive).catch(() => undefined);
  }

  /** Stop polling without leaving (e.g. page unload; the server reaps idle members). */
  stop() {
    this.stopped = true;
    clearTimeout(this.timer);
  }

  private schedule() {
    if (this.stopped) return;
    clearTimeout(this.timer);
    this.timer = setTimeout(() => void this.poll(), POLL_MS);
  }

  private async poll() {
    if (this.stopped) return;
    try {
      await this.timed(() => partiesApi.get(this.id));
      this.failures = 0;
      this.state = "live";
      this.message = null;
    } catch (error) {
      if (
        error instanceof ApiError &&
        (error.status === 403 || error.status === 404 || error.status === 401)
      ) {
        this.close(error, false);
        return;
      }
      this.failures++;
      if (this.failures >= RECONNECT_AFTER) {
        this.state = "reconnecting";
        this.message = errorMessage(error);
      }
    }
    this.schedule();
  }

  private async timed(call: () => Promise<{ party: Party }>) {
    const sent = Date.now();
    const { party } = await call();
    if (this.stopped) return;
    this.clock.add(sent, Date.now(), party.server_now_ms);
    if (this.seen === null || party.revision >= this.seen) {
      this.seen = party.revision;
      this.party = party;
      if (party.item_id !== this.item?.id) void this.loadItem(party.item_id);
    }
  }

  private async loadItem(itemId: string) {
    if (this.loadingItem === itemId) return;
    this.loadingItem = itemId;
    this.itemError = null;
    try {
      const { item } = await itemsApi.get(itemId);
      // A newer title may have been chosen meanwhile.
      if (this.party?.item_id === item.id) this.item = item;
    } catch (error) {
      if (this.party?.item_id === itemId) this.itemError = errorMessage(error);
    } finally {
      if (this.loadingItem === itemId) this.loadingItem = null;
    }
  }

  private close(error: unknown, joining: boolean) {
    this.stop();
    if (error instanceof ApiError && error.status === 404) {
      this.state = "closed";
      this.message = joining
        ? "This party doesn't exist or has ended."
        : "The host ended this party.";
    } else if (error instanceof ApiError && error.status === 403) {
      this.state = "closed";
      this.message = joining
        ? error.message || "You can't join this party."
        : "You no longer have access to this party.";
    } else {
      this.state = "failed";
      this.message = errorMessage(error);
    }
  }
}
