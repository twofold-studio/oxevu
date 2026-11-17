import { fileURLToPath } from "node:url";
import { svelte } from "@sveltejs/vite-plugin-svelte";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig, lazyPlugins } from "vite-plus";

// The Rust server owns /api and /healthz. In development Vite serves the UI on
// 5173 and forwards those paths so the session cookie stays same-origin.
const apiTarget = process.env.OXEVU_API_URL ?? "http://localhost:3000";

export default defineConfig({
  fmt: {},
  lint: {
    ignorePatterns: ["src/lib/api/generated/**"],
    jsPlugins: [{ name: "vite-plus", specifier: "vite-plus/oxlint-plugin" }],
    rules: {
      "vite-plus/prefer-vite-plus-imports": "error",
      "no-restricted-globals": [
        "error",
        {
          name: "fetch",
          message:
            "Use the generated REST SDK. Native media reads belong in the explicit allowlist.",
        },
      ],
    },
    overrides: [
      {
        files: [
          "src/lib/api/sdk.ts",
          "src/lib/playback/playback-error.ts",
          "src/lib/playback/subtitles.svelte.ts",
        ],
        rules: { "no-restricted-globals": "off" },
      },
    ],
    // tsgolint cannot resolve exports from .svelte modules; svelte-check
    // (`pnpm check`) owns type checking instead.
    options: { typeAware: true, typeCheck: false },
  },
  resolve: {
    alias: { $lib: fileURLToPath(new URL("./src/lib", import.meta.url)) },
  },
  server: {
    port: 5173,
    strictPort: true,
    proxy: {
      "/api": { target: apiTarget, changeOrigin: false },
      "/healthz": { target: apiTarget, changeOrigin: false },
    },
  },
  plugins: lazyPlugins(() => [tailwindcss(), svelte()]),
});
