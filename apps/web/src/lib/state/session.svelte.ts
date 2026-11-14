import { authApi } from "$lib/api/endpoints";
import { setUnauthorizedListener, errorMessage } from "$lib/api/client";
import type { Credentials, User } from "$lib/api/types";
import { catalog } from "./catalog.svelte";

type SessionPhase = "loading" | "unreachable" | "setup" | "signed-out" | "signed-in";

class Session {
  phase = $state<SessionPhase>("loading");
  user = $state<User | null>(null);
  error = $state<string | null>(null);

  constructor() {
    setUnauthorizedListener(() => this.expire());
  }

  async load() {
    this.phase = "loading";
    this.error = null;
    try {
      const status = await authApi.status();
      this.user = status.user;
      this.phase = status.setup_required ? "setup" : status.user ? "signed-in" : "signed-out";
    } catch (error) {
      this.error = errorMessage(error);
      this.phase = "unreachable";
    }
  }

  /**
   * Re-read the signed-in user without blanking the app, so role and
   * permission changes made elsewhere show up in navigation.
   */
  async refresh() {
    if (this.phase !== "signed-in") return;
    try {
      const status = await authApi.status();
      if (status.user) this.user = status.user;
      else this.expire();
    } catch {
      // Keep the current view; real requests report their own errors.
    }
  }

  /** Pass `setupToken` to create the first administrator instead of signing in. */
  async signIn(credentials: Credentials, setupToken?: string) {
    const { user } =
      setupToken === undefined
        ? await authApi.login(credentials)
        : await authApi.setup(credentials, setupToken);
    // Another account (or changed access) sees a different catalog.
    catalog.reset();
    this.user = user;
    this.error = null;
    this.phase = "signed-in";
  }

  async signOut() {
    try {
      await authApi.logout();
    } finally {
      this.user = null;
      this.phase = "signed-out";
    }
  }

  private expire() {
    if (this.phase !== "signed-in") return;
    this.user = null;
    this.error = "Your session has ended. Sign in again.";
    this.phase = "signed-out";
  }
}

export const session = new Session();
