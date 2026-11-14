# Gitadel UI reference

Captured 2026-11-14 at 1440 × 1000 from http://vd.netbird.cloud:3030 using a browser. The original captures below are public, signed-out pages. A second authenticated dark-mode pass is documented below. The captured light theme reflects this browser's initial state, not a user preference for light mode.

## Observed patterns

- Persistent header with brand, breadcrumbs and global search. Small keyboard shortcut hints make search discoverable.
- Narrow icon navigation rail with soft square active states. Repository tools occupy the rail while secondary details occupy a right panel.
- Thin neutral borders organize large areas. Most surfaces have similar colors; grouping comes from alignment and separators rather than shadows.
- Dense lists have a clear title, secondary metadata and aligned actions. Explore uses joined rows within one frame instead of many floating cards.
- Context remains visible while switching repository tabs. Source views add a file tree and toolbar without replacing the overall shell.
- Strong black/white primary actions, outlined secondary controls and small icon actions. Color mostly communicates status, language or identity.
- Compact sans-serif interface text, monospaced technical values and uppercase secondary section labels.
- Empty states include an icon, short title and explanation inside the normal page shell.
- Authentication deliberately changes mood: dark split screen, sculptural background on the left and a narrow, calm form on the right.
- Long documentation remains readable through a constrained content column and clear heading hierarchy.

## Applying this to Oxevu

Use a persistent header with breadcrumbs, global media search and account controls. The desktop rail starts with Home, Movies and Series; administration appears only for authorized users. Expand navigation labels when needed and provide accessible names/tooltips for icon controls.

Carry the restrained surfaces, borders, spacing and button hierarchy into both themes. Artwork supplies most of the color in library views. Use a poster grid for browsing and joined rows for episodes, activity and settings. Reserve the right detail panel for relevant metadata and playback information; collapse it on narrow screens.

Use Oxevu's orange sparingly for identity, selection and progress. Keep success/warning/failure meanings separate. A media detail page should have one clear Play/Resume action, with tracks and metadata secondary. The player should let the video occupy the space and reveal controls on interaction.

Map implementation to shadcn-svelte Sidebar, Breadcrumb, Command, Button, Tooltip, Tabs, Table, Sheet, Dialog, Empty and Skeleton. Style through semantic Tailwind v4 theme tokens. On mobile use a sheet or compact primary navigation; preserve touch targets rather than squeezing the desktop columns.

These are proposed translations of the observed reference, not proof of the user's preference for every detail. Mobile, populated issues and individual commits still need additional inspection.

## Captures

1. [Explore](01-explore.png): joined repository rows and activity summaries.
2. [Repository](02-repository.png): rail, file tree, content and details panel.
3. [History](03-history.png): grouped timeline with status and metadata.
4. [Actions](04-actions.png): compact rows and plain failure messages.
5. [Issues](05-issues.png): search/filter toolbar and empty state.
6. [Releases](06-releases.png): page heading and empty state.
7. [Tags](07-tags.png): minimal aligned list.
8. [Registry](08-registry.png): nested table and inline copy controls.
9. [Profile](09-profile.png): pinned content and contribution grid.
10. [Changelog](10-changelog.png): long-form reading layout.
11. [API](11-api.png): API documentation navigation and endpoint layout.
12. [Login](12-login.png): dark split composition.
13. [Source](13-source.png): toolbar and code density.
14. [Installation](14-installation.png): document preview within the file shell.
15. [Chorus](15-chorus.png): same repository structure with another identity.

## Authenticated dark-mode pass

Signed in successfully on vd with the saved Gitadel - Neo credential after the user explicitly approved its use on this origin. Selected Dark through the account theme menu. The user explicitly requests dark mode for Oxevu; this supersedes the original light captures as the implementation reference.

Captured 18 authenticated views under `dark/` at 1280 × 633. The original login remains the dark authentication reference; it was not recaptured while authenticated. The other 14 original routes were recaptured, plus:

- [Account profile](dark/16-account.png): horizontal account tabs and a joined settings frame; labels/explanations on the left and controls on the right.
- [Administration appearance](dark/17-admin-general.png): two levels of underlined tabs and a restrained form panel.
- [Storage overview](dark/18-admin-storage.png): readable disk usage bars, numeric labels and storage breakdowns inside the ordinary application shell.
- [Administration activity](dark/19-admin-activity.png): compact event rows, muted actor/context and right-aligned timestamps.
- [Repository in dark mode](dark/02-repository.png): black base, very subtle surface changes, hairline separators and off-white text.

The signed-in header adds a compact New action and avatar. Active rail items use a dark rounded background and a bright edge marker. Primary controls are off-white; supporting controls are dark and outlined. Apply these relationships through semantic tokens rather than copying isolated hex colors.

For Oxevu, use the grouped settings layout for library roots and transcoding configuration, the storage presentation for cache/media capacity, and the activity rows for scan/transcode jobs. Keep consumer browsing focused on artwork and playback, with administration using this denser reference. The screenshots are reference artifacts, not assets to ship in the app.
