<script lang="ts">
  import SearchIcon from "@lucide/svelte/icons/search";
  import * as InputGroup from "$lib/components/ui/input-group";
  import { Kbd } from "$lib/components/ui/kbd";
  import { href, router } from "$lib/state/router.svelte";
  import { cn } from "$lib/utils";

  let { class: className }: { class?: string } = $props();

  let input = $state<HTMLInputElement | null>(null);
  // Mirror the query of the search page so the box shows what is being searched.
  let query = $state(router.route.name === "search" ? router.route.query : "");

  function submit(event: SubmitEvent) {
    event.preventDefault();
    const trimmed = query.trim();
    if (trimmed) router.go(href.search(trimmed));
  }

  function onWindowKeydown(event: KeyboardEvent) {
    if (event.key !== "/" || event.metaKey || event.ctrlKey || event.altKey) return;
    const target = event.target as HTMLElement | null;
    if (target?.closest("input, textarea, select, [contenteditable=true]")) return;
    event.preventDefault();
    input?.focus();
  }
</script>

<svelte:window onkeydown={onWindowKeydown} />

<form role="search" onsubmit={submit} class={cn("w-full", className)}>
  <InputGroup.Root class="h-9">
    <InputGroup.Addon>
      <SearchIcon />
    </InputGroup.Addon>
    <InputGroup.Input
      bind:ref={input}
      bind:value={query}
      type="search"
      placeholder="Search titles"
      aria-label="Search titles"
      autocomplete="off"
      enterkeyhint="search"
    />
    <InputGroup.Addon align="inline-end" class="hidden sm:flex">
      <Kbd>/</Kbd>
    </InputGroup.Addon>
  </InputGroup.Root>
</form>
