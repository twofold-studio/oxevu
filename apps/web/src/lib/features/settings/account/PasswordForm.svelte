<script lang="ts">
  import { toast } from "svelte-sonner";
  import * as Field from "$lib/components/ui/field";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { authApi } from "$lib/api/endpoints";
  import { errorMessage } from "$lib/api/client";
  import { MIN_PASSWORD_BYTES } from "$lib/api/types";
  import { validatePassword } from "../users/user-form";

  let current = $state("");
  let next = $state("");
  let confirm = $state("");
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const errors = $derived({
    current: current ? null : "Enter your current password.",
    next: validatePassword(next, true) ?? (next === current ? "Choose a different password." : null),
    confirm: confirm === next ? null : "The passwords don't match.",
  });
  const show = (message: string | null) => (submitted ? message : null);

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    failure = null;
    if (errors.current || errors.next || errors.confirm || pending) return;
    pending = true;
    try {
      await authApi.changePassword({ current_password: current, new_password: next });
      current = next = confirm = "";
      submitted = false;
      toast.success("Password changed.");
    } catch (error) {
      failure = errorMessage(error);
    } finally {
      pending = false;
    }
  }
</script>

<form onsubmit={submit} novalidate>
  <Field.Group>
    <Field.Field data-invalid={show(errors.current) ? true : undefined}>
      <Field.Label for="current-password">Current password</Field.Label>
      <Input id="current-password" type="password" autocomplete="current-password" bind:value={current} aria-invalid={show(errors.current) ? true : undefined} />
      {#if show(errors.current)}<Field.Error>{errors.current}</Field.Error>{/if}
    </Field.Field>
    <Field.Field data-invalid={show(errors.next) ? true : undefined}>
      <Field.Label for="new-password">New password</Field.Label>
      <Input id="new-password" type="password" autocomplete="new-password" bind:value={next} aria-invalid={show(errors.next) ? true : undefined} />
      {#if show(errors.next)}
        <Field.Error>{errors.next}</Field.Error>
      {:else}
        <Field.Description>At least {MIN_PASSWORD_BYTES} characters.</Field.Description>
      {/if}
    </Field.Field>
    <Field.Field data-invalid={show(errors.confirm) ? true : undefined}>
      <Field.Label for="confirm-password">Confirm new password</Field.Label>
      <Input id="confirm-password" type="password" autocomplete="new-password" bind:value={confirm} aria-invalid={show(errors.confirm) ? true : undefined} />
      {#if show(errors.confirm)}<Field.Error>{errors.confirm}</Field.Error>{/if}
    </Field.Field>
    {#if failure}<p class="text-sm text-destructive" role="alert">{failure}</p>{/if}
    <div>
      <Button type="submit" disabled={pending}>
        {#if pending}<Spinner data-icon="inline-start" />{/if}
        Change password
      </Button>
    </div>
  </Field.Group>
</form>
