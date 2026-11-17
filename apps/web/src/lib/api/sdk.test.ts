import { afterEach, describe, expect, it, vi } from "vite-plus/test";
import { authApi, librariesApi } from "./endpoints";
import { ApiError, setUnauthorizedListener } from "./client";

afterEach(() => {
  vi.unstubAllGlobals();
  setUnauthorizedListener(null);
});

describe("generated API transport", () => {
  it("sends typed login bodies with same-origin credentials", async () => {
    const fetch = vi.fn(async (_input: RequestInfo | URL) =>
      Response.json({ user: { id: "owner" } }),
    );
    vi.stubGlobal("fetch", fetch);
    await authApi.login({ username: "owner", password: "long-enough-password" });
    const request = fetch.mock.calls[0]?.[0] as Request | undefined;
    expect(request?.url).toBe("http://localhost/api/v1/auth/login");
    expect(request?.credentials).toBe("same-origin");
    expect(await request?.json()).toEqual({ username: "owner", password: "long-enough-password" });
  });

  it("expires a session for authenticated calls but not invalid login", async () => {
    vi.stubGlobal(
      "fetch",
      vi.fn(async () => Response.json({ error: "sign in required" }, { status: 401 })),
    );
    const expired = vi.fn();
    setUnauthorizedListener(expired);
    await expect(
      authApi.login({ username: "owner", password: "wrong-password" }),
    ).rejects.toBeInstanceOf(ApiError);
    expect(expired).not.toHaveBeenCalled();
    await expect(librariesApi.list()).rejects.toMatchObject({ status: 401 });
    expect(expired).toHaveBeenCalledOnce();
  });
});
