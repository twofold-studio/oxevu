<script lang="ts">
  import PencilIcon from "@lucide/svelte/icons/pencil";
  import { Badge } from "$lib/components/ui/badge";
  import { Button } from "$lib/components/ui/button";
  import type { UserSettings } from "$lib/api/admin-types";
  import type { Library } from "$lib/api/types";

  let {
    user,
    self,
    libraries,
    onedit,
  }: { user: UserSettings; self: boolean; libraries: Library[]; onedit: () => void } = $props();

  const access = $derived.by(() => {
    if (user.is_admin || user.all_libraries) return "All libraries";
    const names = libraries.filter((l) => user.library_ids.includes(l.id)).map((l) => l.name);
    return names.length ? names.join(", ") : "No libraries";
  });

  const permissions = $derived(
    [
      user.can_play ? null : "Can't play",
      user.can_play && !user.can_transcode ? "Original files only" : null,
      user.syncplay_access === "create_join"
        ? "Hosts parties"
        : user.syncplay_access === "join"
          ? "Joins parties"
          : "No parties",
    ].filter(Boolean),
  );
</script>

<li class="flex flex-wrap items-center gap-x-4 gap-y-2 px-5 py-4">
  <div class="flex min-w-0 flex-1 flex-col gap-1">
    <div class="flex flex-wrap items-center gap-2">
      <span class="truncate font-medium">{user.username}</span>
      {#if self}<span class="text-xs text-muted-foreground">(you)</span>{/if}
      {#if user.is_admin}<Badge variant="outline">Administrator</Badge>{/if}
      {#if user.disabled}<Badge variant="destructive">Disabled</Badge>{/if}
    </div>
    <p class="truncate text-xs text-muted-foreground">{access} · {permissions.join(" · ")}</p>
  </div>
  <Button variant="outline" size="sm" onclick={onedit}>
    <PencilIcon data-icon="inline-start" />
    Edit
  </Button>
</li>
