<script lang="ts">
  import type { SavePhase } from "../settings-form.svelte";

  // Page-level autosave feedback, announced politely to screen readers.
  let {
    phase,
    failure,
    invalid,
    saved = "Changes saved.",
  }: { phase: SavePhase; failure: string | null; invalid: boolean; saved?: string } = $props();
</script>

<p class="text-sm text-muted-foreground" role="status" aria-live="polite">
  {#if failure}<span class="text-destructive">Not saved: {failure}. Change a field to retry.</span>
  {:else if invalid}Fix the highlighted fields to save.
  {:else if phase === "pending" || phase === "saving"}Saving changes…
  {:else if phase === "saved"}{saved}
  {:else}Changes save automatically.{/if}
</p>
