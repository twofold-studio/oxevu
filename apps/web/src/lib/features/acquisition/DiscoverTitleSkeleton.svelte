<script lang="ts">
  import { Skeleton } from "$lib/components/ui/skeleton";

  let { series = false }: { series?: boolean } = $props();
</script>

<div class="grid items-start gap-7 md:grid-cols-[minmax(12rem,20rem)_minmax(0,1fr)]" aria-hidden="true">
  <Skeleton class="aspect-[2/3] w-36 max-w-80 rounded-xl md:w-full" />
  <div class="flex min-w-0 flex-col gap-5">
    <Skeleton class="h-10 w-2/3" />
    <Skeleton class="h-5 w-48 max-w-full" />
    <div class="grid gap-3">
      <Skeleton class="h-4 w-full" />
      <Skeleton class="h-4 w-5/6" />
      <Skeleton class="h-4 w-2/3" />
    </div>
    <Skeleton class="h-9 w-28" />
    {#if series}
      <div class="grid gap-4 rounded-xl border p-4">
        <Skeleton class="h-5 w-24" />
        {#each [0, 1, 2] as row (row)}
          <div class="flex items-center justify-between gap-4">
            <Skeleton class="h-4 w-24" />
            <Skeleton class="h-8 w-16" />
          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>
