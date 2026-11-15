<script lang="ts">
  import StarIcon from "@lucide/svelte/icons/star";
  import { ratingLabel, type Rating } from "$lib/media/metadata";
  import ProviderMark from "./ProviderMark.svelte";

  // Source mark (only when stored), a star and the score. Screen readers get
  // one full sentence instead of the abbreviated visual parts.
  let { rating }: { rating: Rating } = $props();
  const label = $derived(ratingLabel(rating));
</script>

<span class="inline-flex items-center gap-1.5" title={label}>
  <span class="sr-only">{label}</span>
  <span aria-hidden="true" class="inline-flex items-center gap-1.5">
    {#if rating.source}<ProviderMark source={rating.source} />{/if}
    <StarIcon class="size-3.5 fill-rating text-rating" />
    <span class="font-medium text-foreground tabular-nums">{rating.value.toFixed(1)}</span>  </span>
</span>
