<script lang="ts">
  import { Skeleton } from "$lib/components/ui/skeleton";
  import PlateGrid from "./PlateGrid.svelte";

  let { count = 8 }: { count?: number } = $props();
</script>

<PlateGrid label="Loading">
  {#each Array.from({ length: count }, (_, i) => i) as i (i)}
    <li class="flex aspect-[4/3] flex-col justify-between border-r border-b p-4 sm:p-5">
      <Skeleton class="h-3 w-20" />
      <div class="flex flex-col gap-2">
        <Skeleton class="h-6 w-4/5" />
        <Skeleton class="h-3 w-1/3" />
      </div>
    </li>
  {/each}
</PlateGrid>
