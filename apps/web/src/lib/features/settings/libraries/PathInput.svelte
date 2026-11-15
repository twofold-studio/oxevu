<script lang="ts">
  import { onDestroy } from "svelte";
  import { Input } from "$lib/components/ui/input";
  import { librariesApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import { cn } from "$lib/utils";

  let { value = $bindable(""), label, invalid = false }: {
    value: string;
    label: string;
    invalid?: boolean;
  } = $props();
  const id = $props.id();
  let suggestions = $state<{ name: string; path: string }[]>([]);
  let status = $state("");
  let focused = $state(false);
  let active = $state(-1);
  const expanded = $derived(focused && suggestions.length > 0);
  let timer: ReturnType<typeof setTimeout> | undefined;
  let generation = 0;

  function schedule() {
    clearTimeout(timer);
    const current = ++generation;
    suggestions = [];
    active = -1;
    status = "";
    if (!value.startsWith("/")) return;
    const path = value;
    timer = setTimeout(async () => {
      status = "Looking for folders…";
      try {
        const result = await librariesApi.suggestPaths(path);
        if (current !== generation) return;
        suggestions = result.paths;
        status = result.truncated ? "Keep typing to narrow the folders." :
          result.paths.length ? "Choose a folder or keep typing." : "No matching folders.";
      } catch (error) {
        if (current === generation) status = errorMessage(error);
      }
    }, 200);
  }

  function choose(path: string) {
    value = path;
    schedule();
  }

  function keydown(event: KeyboardEvent) {
    if (event.key === "Escape" && expanded) {
      event.preventDefault();
      event.stopPropagation();
      focused = false;
    } else if (expanded && (event.key === "ArrowDown" || event.key === "ArrowUp")) {
      event.preventDefault();
      active = (active + (event.key === "ArrowDown" ? 1 : -1) + suggestions.length) % suggestions.length;
    } else if (expanded && event.key === "Enter" && active >= 0) {
      event.preventDefault();
      choose(suggestions[active].path);
    }
  }

  onDestroy(() => { clearTimeout(timer); generation++; });
</script>

<div class="relative flex min-w-0 flex-1 flex-col gap-1">
  <Input
    bind:value={() => value, (next) => { value = String(next ?? ""); focused = true; schedule(); }}
    onfocus={() => { focused = true; schedule(); }}
    onblur={() => (focused = false)}
    onkeydown={keydown}
    role="combobox"
    aria-autocomplete="list"
    aria-expanded={expanded}
    aria-controls={`${id}-folders`}
    aria-activedescendant={expanded && active >= 0 ? `${id}-folder-${active}` : undefined}
    placeholder="/srv/media/movies"
    class="font-mono"
    spellcheck={false}
    autocapitalize="none"
    autocomplete="off"
    aria-label={label}
    aria-invalid={invalid || undefined}
    aria-describedby={`${id}-status`}
  />
  {#if expanded}
    <div id={`${id}-folders`} role="listbox" aria-label="Matching server folders"
      class="absolute inset-x-0 top-9 z-10 max-h-48 overflow-y-auto rounded-lg border bg-popover p-1 shadow-md">
      {#each suggestions as folder, index (folder.path)}
        <button id={`${id}-folder-${index}`} type="button" role="option" aria-selected={active === index} title={folder.path}
          class={cn("block w-full rounded px-2 py-1.5 text-left font-mono text-sm break-all hover:bg-accent", active === index && "bg-accent text-accent-foreground")}
          onpointerdown={(event) => event.preventDefault()} onclick={() => choose(folder.path)}>{folder.name}/</button>
      {/each}
    </div>
  {/if}
  <p id={`${id}-status`} class="text-xs text-muted-foreground" role="status" aria-live="polite">{status}</p>
</div>
