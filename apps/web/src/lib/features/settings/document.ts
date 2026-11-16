// One server settings record edited from several pages. The server only
// accepts complete payloads (`PUT /settings`, `PUT /acquisition/settings`), so
// each page sends just the fields it owns and this merges them onto the latest
// copy the server returned. Writes run one at a time and a load waits for
// queued writes, so a page opened while another page is still saving never
// starts from, or later writes back, an older copy.
//
// `E` is the server response (it may carry read-only extras such as probed
// capabilities); `T` is the editable record inside it.

export class SettingsDocument<E, T extends object> {
  private latest: E | null = null;
  private queue: Promise<unknown> = Promise.resolve();

  constructor(
    private readonly fetch: () => Promise<E>,
    private readonly save: (value: T) => Promise<E>,
    readonly settingsOf: (envelope: E) => T,
  ) {}

  /** Fresh server copy, after any writes already queued from other pages. */
  async load(): Promise<E> {
    const run = this.queue
      .catch(() => undefined)
      .then(async () => {
        const envelope = await this.fetch();
        this.latest = envelope;
        return structuredClone(envelope);
      });
    // Reads also hold the queue: a slow page load must not replace a newer
    // write response in `latest` after that write has already completed.
    this.queue = run;
    return run;
  }

  /** Queue a write of `fields` merged onto the newest known copy. */
  patch(fields: Partial<T>): Promise<E> {
    const run = this.queue
      .catch(() => undefined)
      .then(async () => {
        const base = this.settingsOf(this.latest ?? (await this.fetch()));
        const envelope = await this.save({ ...base, ...fields });
        this.latest = envelope;
        return structuredClone(envelope);
      });
    this.queue = run;
    return run;
  }
}

export function pick<T extends object, K extends keyof T>(
  value: T,
  keys: readonly K[],
): Pick<T, K> {
  const picked = {} as Pick<T, K>;
  for (const key of keys) picked[key] = value[key];
  return picked;
}
