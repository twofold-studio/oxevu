<script lang="ts">
  import { Skeleton } from "$lib/components/ui/skeleton";
  import PlateGrid from "./PlateGrid.svelte";

  let { count = 12 }: { count?: number } = $props();
</script>

<PlateGrid label="Loading">
  {#each Array.from({ length: count }, (_, i) => i) as i (i)}
    <li class="flex flex-col gap-2">
      <Skeleton class="aspect-[2/3] rounded-lg" />
      <div class="flex flex-col gap-1.5">
        <Skeleton class="h-3.5 w-4/5" />
        <Skeleton class="h-3 w-1/3" />
      </div>
    </li>
  {/each}
</PlateGrid>
