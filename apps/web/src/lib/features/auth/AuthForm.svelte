<script lang="ts">
  import EyeIcon from "@lucide/svelte/icons/eye";
  import EyeOffIcon from "@lucide/svelte/icons/eye-off";
  import * as Field from "$lib/components/ui/field";
  import * as InputGroup from "$lib/components/ui/input-group";
  import * as Alert from "$lib/components/ui/alert";
  import { Input } from "$lib/components/ui/input";
  import { Button } from "$lib/components/ui/button";
  import { Spinner } from "$lib/components/ui/spinner";
  import { session } from "$lib/state/session.svelte";
  import { ApiError, errorMessage } from "$lib/api/client";
  import { MIN_PASSWORD_BYTES } from "$lib/api/types";

  let { mode, notice }: { mode: "setup" | "sign-in"; notice: string | null } = $props();

  const setup = $derived(mode === "setup");

  // The setup token lives only in this component's state and is dropped as
  // soon as the request settles; it is never written to storage.
  let setupToken = $state("");
  let username = $state("");
  let password = $state("");
  let confirm = $state("");
  let showPassword = $state(false);
  let submitted = $state(false);
  let pending = $state(false);
  let failure = $state<string | null>(null);

  const passwordBytes = $derived(new TextEncoder().encode(password).length);
  const errors = $derived({
    token: setup && !setupToken.trim() ? "Enter the setup token from the server." : null,
    username: !username.trim() ? "Enter a username." : null,
    password: !password
      ? "Enter a password."
      : setup && passwordBytes < MIN_PASSWORD_BYTES
        ? `Use at least ${MIN_PASSWORD_BYTES} characters.`
        : null,
    confirm: setup && confirm !== password ? "Passwords don't match." : null,
  });
  const valid = $derived(!Object.values(errors).some(Boolean));

  async function submit(event: SubmitEvent) {
    event.preventDefault();
    submitted = true;
    failure = null;
    if (!valid || pending) return;
    pending = true;
    try {
      const credentials = { username: username.trim(), password };
      await session.signIn(credentials, setup ? setupToken.trim() : undefined);
    } catch (error) {
      failure = describeFailure(error);
      // Cleared fields shouldn't light up as "required" after a server error.
      submitted = false;
    } finally {
      pending = false;
      setupToken = "";
      password = "";
      confirm = "";
    }
  }

  function describeFailure(error: unknown): string {
    if (!(error instanceof ApiError)) return errorMessage(error);
    // Mutations are rejected with 403 when the browser Origin doesn't match
    // the server's configured OXEVU_ORIGIN.
    if (error.status === 403 && /origin/i.test(error.message)) {
      return `The server rejected this page's address (${window.location.origin}). Set OXEVU_ORIGIN on the server to match it.`;
    }
    if (setup && error.status === 403) return "That setup token isn't valid. Copy it again from the server; it changes each time Oxevu starts.";
    // Bad credentials currently come back as 400 "invalid credentials".
    if (!setup && (error.status === 401 || /credential/i.test(error.message))) {
      return "Username or password is incorrect.";
    }
    return error.message;
  }

  const show = (message: string | null) => (submitted ? message : null);
</script>

<div class="flex flex-col gap-2">
  <h1 class="text-2xl font-semibold tracking-tight">{setup ? "Create the administrator" : "Sign in"}</h1>
  <p class="text-sm leading-6 text-muted-foreground">
    {#if setup}
      This server has no accounts yet. The first account can add and scan libraries.
    {:else}
      Use your Oxevu account to browse and play your library.
    {/if}
  </p>
</div>

{#if failure ?? notice}
  <Alert.Root variant="destructive">
    <Alert.Description>{failure ?? notice}</Alert.Description>
  </Alert.Root>
{/if}

<form onsubmit={submit} novalidate>
  <Field.Group>
    {#if setup}
      <Field.Field data-invalid={show(errors.token) ? true : undefined}>
        <Field.Label for="setup-token">Setup token</Field.Label>
        <Input
          id="setup-token"
          bind:value={setupToken}
          type="password"
          autocomplete="off"
          spellcheck={false}
          class="font-mono"
          aria-invalid={show(errors.token) ? true : undefined}
        />
        <Field.Description>
          On the server, read <code class="font-mono text-foreground">setup-token</code> in the
          <code class="font-mono text-foreground">OXEVU_DATA_DIR</code> directory. It works once and is not saved here.
        </Field.Description>
        {#if show(errors.token)}<Field.Error>{errors.token}</Field.Error>{/if}
      </Field.Field>
    {/if}

    <Field.Field data-invalid={show(errors.username) ? true : undefined}>
      <Field.Label for="username">Username</Field.Label>
      <Input
        id="username"
        bind:value={username}
        autocomplete="username"
        autocapitalize="none"
        spellcheck={false}
        aria-invalid={show(errors.username) ? true : undefined}
      />
      {#if show(errors.username)}<Field.Error>{errors.username}</Field.Error>{/if}
    </Field.Field>

    <Field.Field data-invalid={show(errors.password) ? true : undefined}>
      <Field.Label for="password">Password</Field.Label>
      <InputGroup.Root>
        <InputGroup.Input
          id="password"
          bind:value={password}
          type={showPassword ? "text" : "password"}
          autocomplete={setup ? "new-password" : "current-password"}
          aria-invalid={show(errors.password) ? true : undefined}
        />
        <InputGroup.Addon align="inline-end">
          <InputGroup.Button
            size="icon-xs"
            aria-label={showPassword ? "Hide password" : "Show password"}
            aria-pressed={showPassword}
            onclick={() => (showPassword = !showPassword)}
          >
            {#if showPassword}<EyeOffIcon />{:else}<EyeIcon />{/if}
          </InputGroup.Button>
        </InputGroup.Addon>
      </InputGroup.Root>
      {#if setup && !show(errors.password)}
        <Field.Description>At least {MIN_PASSWORD_BYTES} characters.</Field.Description>
      {/if}
      {#if show(errors.password)}<Field.Error>{errors.password}</Field.Error>{/if}
    </Field.Field>

    {#if setup}
      <Field.Field data-invalid={show(errors.confirm) ? true : undefined}>
        <Field.Label for="confirm">Confirm password</Field.Label>
        <Input
          id="confirm"
          bind:value={confirm}
          type={showPassword ? "text" : "password"}
          autocomplete="new-password"
          aria-invalid={show(errors.confirm) ? true : undefined}
        />
        {#if show(errors.confirm)}<Field.Error>{errors.confirm}</Field.Error>{/if}
      </Field.Field>
    {/if}

    <Button type="submit" size="lg" class="w-full" disabled={pending}>
      {#if pending}<Spinner data-icon="inline-start" />{/if}
      {setup ? "Create administrator" : "Sign in"}
    </Button>
  </Field.Group>
</form>

{#if !setup}
  <p class="text-center text-sm text-muted-foreground">There is no self sign-up. Ask the person who runs this server for an account.</p>
{/if}
