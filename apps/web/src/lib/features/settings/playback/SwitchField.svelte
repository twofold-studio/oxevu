<script lang="ts">
  import * as Field from "$lib/components/ui/field";
  import { Switch } from "$lib/components/ui/switch";

  let {
    id,
    label,
    description,
    checked = $bindable(),
    disabled = false,
    error,
  }: {
    id: string;
    label: string;
    description: string;
    checked: boolean;
    disabled?: boolean;
    error?: string | null;
  } = $props();
</script>

<Field.Field orientation="horizontal" data-disabled={disabled ? true : undefined} data-invalid={error ? true : undefined}>
  <Switch {id} bind:checked {disabled} aria-invalid={error ? true : undefined} />
  <Field.Content>
    <Field.Label for={id}>{label}</Field.Label>
    <Field.Description>{description}</Field.Description>
    {#if error}<Field.Error>{error}</Field.Error>{/if}
  </Field.Content>
</Field.Field>
