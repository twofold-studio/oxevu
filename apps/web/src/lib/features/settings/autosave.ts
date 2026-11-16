/** Debounced, serialized writes. Only the latest pending snapshot is sent. */
export class Autosave<T> {
  private pending: T | null = null;
  private timer: ReturnType<typeof setTimeout> | undefined;
  private running = false;
  private saved = "";

  constructor(
    private readonly write: (value: T) => Promise<void>,
    private readonly status: (state: "saving" | "saved" | "error", error?: unknown) => void,
    private readonly delay = 500,
  ) {}

  /** An edit is waiting for the debounce or for the current write. */
  get queued() {
    return this.pending !== null;
  }

  reset(value: T) {
    this.saved = JSON.stringify(value);
  }

  update(value: T | null) {
    clearTimeout(this.timer);
    this.pending =
      value === null || JSON.stringify(value) === this.saved ? null : structuredClone(value);
    if (this.pending !== null) this.timer = setTimeout(() => void this.flush(), this.delay);
  }

  async flush() {
    clearTimeout(this.timer);
    if (this.running) return;
    this.running = true;
    try {
      while (this.pending !== null) {
        const value = this.pending;
        this.pending = null;
        this.status("saving");
        try {
          await this.write(value);
          this.saved = JSON.stringify(value);
          this.status("saved");
        } catch (error) {
          this.status("error", error);
          // Retry only after a new edit; never loop on an unavailable server.
          break;
        }
      }
    } finally {
      this.running = false;
    }
  }
}
