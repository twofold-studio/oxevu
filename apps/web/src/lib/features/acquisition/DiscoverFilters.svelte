<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import * as Select from "$lib/components/ui/select";
  import { Input } from "$lib/components/ui/input";
  import type { Genre, DiscoverSort } from "$lib/api/generated/types.gen";
  import type { DiscoverFilters } from "$lib/media/discover-filters";

  let { value = $bindable(), genres, searching }: { value: DiscoverFilters; genres: Genre[]; searching: boolean } = $props();
  const options = $derived([
    { key: "genre" as const, label: "Genre", items: genres.map(g => ({ value: String(g.id), label: g.name })) },
    { key: "min_rating" as const, label: "Minimum rating", items: [5,6,7,8,9].map(n => ({value: String(n), label: `${n}+ / 10`})) },
    { key: "language" as const, label: "Original language", items: [ ["en","English"], ["bg","Bulgarian"], ["es","Spanish"], ["fr","French"], ["de","German"], ["ja","Japanese"], ["ko","Korean"], ["zh","Chinese"], ["hi","Hindi"], ["ru","Russian"], ["it","Italian"] ].map(([value,label]) => ({value,label})) },
    { key: "sort" as const, label: "Sort by", items: [{value:"popular",label:"Most popular"},{value:"rating",label:"Highest rated"},{value:"newest",label:"Newest releases"},{value:"oldest",label:"Oldest releases"}] },
  ]);
  function change(key: "genre" | "min_rating" | "language" | "sort", raw: string) {
    const next = { ...value };
    if (raw === "any") delete next[key];
    else if (key === "genre" || key === "min_rating") next[key] = Number(raw);
    else if (key === "sort") next.sort = raw as DiscoverSort;
    else next.language = raw;
    value = next;
  }
</script>

<Field.FieldGroup class="grid grid-cols-2 gap-3 lg:grid-cols-5">
  <Field.Field><Field.Label for="discover-year">Release year</Field.Label><Input id="discover-year" type="number" min={1880} max={2100} step={1} placeholder="Any year" bind:value={value.year} /></Field.Field>
  {#each options as option (option.key)}
    <Field.Field data-disabled={searching}>
      <Field.Label for={`discover-${option.key}`}>{option.label}</Field.Label>
      <Select.Root type="single" value={String(value[option.key] ?? (option.key === "sort" ? "popular" : "any"))} onValueChange={v => change(option.key, v)} disabled={searching} items={option.key === "sort" ? option.items : [{value:"any",label:"Any"}, ...option.items]}>
        <Select.Trigger id={`discover-${option.key}`} class="w-full"><Select.Value /></Select.Trigger>
        <Select.Content><Select.Group>{#if option.key !== "sort"}<Select.Item value="any">Any</Select.Item>{/if}{#each option.items as item (item.value)}<Select.Item value={item.value}>{item.label}</Select.Item>{/each}</Select.Group></Select.Content>
      </Select.Root>
    </Field.Field>
  {/each}
</Field.FieldGroup>
{#if searching}<p class="text-xs text-muted-foreground">Title search supports release year. Clear the search to browse by genre, rating, original language and sort.</p>
{:else}<p class="text-xs text-muted-foreground">Rating filters and highest-rated sorting require at least 100 votes.</p>{/if}
