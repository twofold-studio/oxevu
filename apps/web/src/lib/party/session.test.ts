import { beforeEach, describe, expect, it, vi } from "vite-plus/test";
import type { Party, PartyControl } from "$lib/api/playback-types";

const fake = vi.hoisted(() => ({
  connections: [] as {
    emit: (party: Party) => void;
    control: ReturnType<typeof vi.fn>;
    stopped: boolean;
  }[],
  go: vi.fn(),
  get: vi.fn(),
}));
vi.mock("$lib/api/endpoints", () => ({ itemsApi: { get: fake.get } }));
vi.mock("svelte-sonner", () => ({ toast: { error: vi.fn() } }));
vi.mock("$lib/state/router.svelte", () => ({
  href: { watch: (id: string) => `#/watch/${id}` },
  router: { go: fake.go },
}));
vi.mock("./connection.svelte", () => ({
  PartyConnection: class {
    state = "joining";
    stopped = false;
    party: Party;
    control = vi.fn(async (_body: PartyControl) => {});
    constructor(
      readonly id: string,
      readonly emit: (party: Party) => void,
    ) {
      this.party = {
        id,
        name: "Friday",
        item_id: null,
        paused: true,
        position_seconds: 0,
        host_user_id: "host",
        updated_at_ms: 0,
        server_now_ms: 0,
        revision: 1,
        playback_revision: 0,
        members: [],
      };
      fake.connections.push(this);
    }
    async start() {
      this.state = "live";
      this.emit(this.party);
    }
    leave() {
      this.stopped = true;
    }
    stop() {
      this.stopped = true;
    }
  },
}));

import { PartySession } from "./session.svelte";

beforeEach(() => {
  fake.connections.length = 0;
  fake.go.mockClear();
  fake.get.mockReset();
  const data = new Map<string, string>();
  vi.stubGlobal("sessionStorage", {
    getItem: (key: string) => data.get(key) ?? null,
    setItem: (key: string, value: string) => data.set(key, value),
    removeItem: (key: string) => data.delete(key),
  });
});

describe("tab-wide party membership", () => {
  it("joins without opening a player until a member selects a title", async () => {
    const session = new PartySession();
    await session.join("room");
    expect(fake.go).not.toHaveBeenCalled();
    const connection = fake.connections[0];
    const playing = { ...session.connection!.party!, item_id: "movie", playback_revision: 1 };
    connection.emit(playing);
    expect(fake.go).toHaveBeenLastCalledWith("#/watch/movie");
    fake.go.mockClear();
    // A pause, seek, heartbeat or membership update never drags browsing users back.
    connection.emit({ ...playing, paused: true, revision: 5, position_seconds: 40 });
    expect(fake.go).not.toHaveBeenCalled();
    connection.emit({ ...playing, playback_revision: 2 });
    expect(fake.go).toHaveBeenCalledWith("#/watch/movie");
  });

  it("opening a library title broadcasts its position without local preferences", async () => {
    const session = new PartySession();
    await session.join("room");
    fake.get.mockResolvedValue({ item: { id: "movie", position_seconds: 30, watched: false } });
    await session.play("movie", true);
    expect(fake.connections[0].control).toHaveBeenCalledExactlyOnceWith({
      action: "item",
      item_id: "movie",
      position_seconds: 0,
    });
  });

  it("keeps polling across navigation and ignores responses from replaced connections", async () => {
    const session = new PartySession();
    await session.join("old");
    const old = fake.connections[0];
    await session.join("new");
    expect(old.stopped).toBe(true);
    old.emit({ ...session.connection!.party!, item_id: "old-movie", playback_revision: 1 });
    expect(fake.go).not.toHaveBeenCalled();
    session.leave();
    expect(session.connection).toBeNull();
    expect(fake.connections[1].stopped).toBe(true);
  });
});
