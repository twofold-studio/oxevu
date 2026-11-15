# Overlays

Use centered modals for focused tasks. Keep the same pattern on phones.
Do not introduce right-side drawers or bottom sheets in application flows.

| Need | Component | Example |
| --- | --- | --- |
| Form, search or focused task | `Dialog` | Add library, edit user, identify title |
| Longer search results | `Dialog.Content size="wide"` | Identify title, find subtitles |
| Destructive confirmation | `AlertDialog` | Remove library configuration |
| Short contextual choices | `DropdownMenu` | Quality, subtitle track, row actions |
| Ongoing workflow or deep navigation | Page | Playback settings, season details |
| Feedback without a decision | Inline status or toast | Saved, scan failed |

```svelte
<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header>
      <Dialog.Title>Add a library</Dialog.Title>
      <Dialog.Description>Choose its folders.</Dialog.Description>
    </Dialog.Header>
    <!-- Form fields -->
    <Dialog.Footer><!-- Cancel, then the primary action --></Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
```

Shared components own placement, width, viewport limits, padding, backdrop,
close control and motion. Use the default width for forms; `wide` for results.
Do not add feature-specific positioning or width/height overrides.
The whole modal scrolls when content is taller than the viewport. Retain a
screen gutter of at least 16px, including on phones. Avoid nested scrolling regions.

Every modal has a title and an accessible description. Use the primitive's
focus trap, Escape handling and focus return; do not implement your own.
Keep controls keyboard reachable. Put errors beside the affected input or
action. Use `Dialog.Footer` for actions: secondary first, primary last.
Existing settings autosave; creations, password changes and removals remain
explicit actions. Never add Save/Cancel to an autosaving form.

Open only one modal at a time. Close a menu before starting a modal task.
Player dialogs stay inside the fullscreen player portal; other dialogs use
the normal portal. Keep player controls visible while their menus are open.
Respect reduced motion through the shared theme rules.

Check desktop, 390px width, short viewports, keyboard focus, Escape/close and
focus return. Check destructive actions in a disposable instance.
