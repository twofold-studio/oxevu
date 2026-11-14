<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import * as InputGroup from "$lib/components/ui/input-group";

  // Labelled numeric input with a unit and inline validation.
  let {
    id,
    label,
    description,
    unit,
    value = $bindable(),
    error,
    step = 1,
    min,
    max,
    disabled = false,
  }: {
    id: string;
    label: string;
    description?: string;
    unit?: string;
    value: number;
    error?: string | null;
    step?: number;
    min?: number;
    max?: number;
    disabled?: boolean;
  } = $props();
</script>

<Field.Field data-invalid={error ? true : undefined} data-disabled={disabled ? true : undefined}>
  <Field.Label for={id}>{label}</Field.Label>
  <InputGroup.Root class="max-w-48">
    <InputGroup.Input
      {id}
      type="number"
      inputmode="decimal"
      bind:value
      {step}
      {min}
      {max}
      {disabled}
      class="font-mono tabular-nums"
      aria-invalid={error ? true : undefined}
    />
    {#if unit}
      <InputGroup.Addon align="inline-end"><InputGroup.Text>{unit}</InputGroup.Text></InputGroup.Addon>
    {/if}
  </InputGroup.Root>
  {#if error}
    <Field.Error>{error}</Field.Error>
  {:else if description}
    <Field.Description>{description}</Field.Description>
  {/if}
</Field.Field>
