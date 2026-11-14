import { authApi } from "$lib/api/endpoints";
import { setUnauthorizedListener, errorMessage } from "$lib/api/client";
import type { Credentials, User } from "$lib/api/types";

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

  /** Pass `setupToken` to create the first administrator instead of signing in. */
  async signIn(credentials: Credentials, setupToken?: string) {
    const { user } =
      setupToken === undefined
        ? await authApi.login(credentials)
        : await authApi.setup(credentials, setupToken);
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
