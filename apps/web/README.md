# Oxevu web

Svelte 5 single-page UI for the Oxevu server. Release builds land in `dist/`,
which the server embeds at compile time. There is no Node server in production.

## Development

```sh
pnpm install
pnpm dev          # http://localhost:5173, proxies /api and /healthz to :3000
```

Run the server with the dev origin so its Origin check accepts the UI:

```sh
OXEVU_ORIGIN=http://localhost:5173 cargo run -p oxevu-server
```

Set `OXEVU_API_URL` to proxy to another server address. The proxy keeps the
browser's `Origin` header unchanged.

## Checks

```sh
pnpm check        # svelte-check + tsc
pnpm lint         # oxlint via Vite+
pnpm fmt:check
pnpm build
```

## Layout

- `src/lib/api`: contract types (docs/api-phase-one.md) and the fetch client.
- `src/lib/state`: session, hash router and the shared item catalog.
- `src/lib/media`: grouping, formatting, resume and browser-support helpers.
- `src/lib/features`: auth, home, library browsing, details, player, settings.
- `src/lib/components/ui`: generated shadcn-svelte components.

Routing uses the URL hash, so the server only serves `index.html` and assets.
The setup token is sent once in `X-Oxevu-Setup-Token` and never stored.
