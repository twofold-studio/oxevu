import { describe, expect, test } from "vite-plus/test";
import { serverOrigin } from "./connection";

describe("mobile server origin", () => {
  test("accepts LAN, NetBird and HTTPS origins without storing a path", () => {
    expect(serverOrigin(" http://192.168.0.2:8096/ ")).toBe("http://192.168.0.2:8096");
    expect(serverOrigin("http://vd.netbird.cloud:8096")).toBe("http://vd.netbird.cloud:8096");
    expect(serverOrigin("https://media.example.com")).toBe("https://media.example.com");
    expect(serverOrigin("http://[::1]:8096")).toBe("http://[::1]:8096");
  });
  test.each([
    "javascript:alert(1)", "file:///etc/passwd", "https://user:secret@example.com",
    "https://example.com/api", "https://example.com?token=secret", "https://example.com/#fragment",
    "not a url", "https://example.com:70000",
  ])("rejects credentials, paths and non-server addresses: %s", (input) => {
    expect(() => serverOrigin(input)).toThrow();
  });
});
