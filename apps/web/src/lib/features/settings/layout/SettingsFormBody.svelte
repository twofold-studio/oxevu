<script lang="ts">
  import type { Snippet } from "svelte";
  import ResourceGate from "../ResourceGate.svelte";
  import type { SettingsFormView } from "../settings-form.svelte";
  import SaveStatus from "./SaveStatus.svelte";
  import { SETTINGS_GROUP } from "./group";

  // Loading/forbidden/error states, the framed autosaving form and its status.
  let {
    form,
    what,
    saved,
    children,
  }: { form: SettingsFormView; what: string; saved?: string; children: Snippet } = $props();
</script>

<ResourceGate status={form.status} error={form.loadError} {what} onretry={form.load}>
  {#if form.draft}
    <div class="flex flex-col gap-4">
      <form class={SETTINGS_GROUP} onsubmit={(event) => event.preventDefault()} novalidate>
        {@render children()}
      </form>
      <SaveStatus phase={form.phase} failure={form.failure} invalid={form.invalid} {saved} />
    </div>
  {/if}
</ResourceGate>
