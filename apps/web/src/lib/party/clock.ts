// Estimates the server clock from poll round trips. Each snapshot carries
// `server_now_ms`; assuming the server stamped it halfway through the round
// trip, offset = server_now - local midpoint. The sample with the smallest
// round trip among recent ones is the most trustworthy.

export interface ClockSample {
  offsetMs: number;
  rttMs: number;
}

const WINDOW = 10;

export class ClockEstimator {
  private samples: ClockSample[] = [];

  add(sentAtMs: number, receivedAtMs: number, serverNowMs: number): ClockSample | null {
    const rttMs = receivedAtMs - sentAtMs;
    if (!Number.isFinite(rttMs) || rttMs < 0 || !Number.isFinite(serverNowMs)) return null;
    const sample = { offsetMs: serverNowMs - (sentAtMs + rttMs / 2), rttMs };
    this.samples.push(sample);
    if (this.samples.length > WINDOW) this.samples.shift();
    return sample;
  }

  /** Best current estimate, or null before the first sample. */
  get best(): ClockSample | null {
    let best: ClockSample | null = null;
    for (const sample of this.samples) if (!best || sample.rttMs < best.rttMs) best = sample;
    return best;
  }

  /** Server time now, from local `nowMs`. */
  serverNow(nowMs: number): number {
    return nowMs + (this.best?.offsetMs ?? 0);
  }
}
