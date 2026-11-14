<script lang="ts">
  import OxevuMark from "$lib/components/shell/OxevuMark.svelte";
  import AuthArt from "./AuthArt.svelte";
  import AuthForm from "./AuthForm.svelte";

  let { mode, notice }: { mode: "setup" | "sign-in"; notice: string | null } = $props();
</script>

<!-- Authentication is the one deliberately atmospheric screen (Gitadel login recipe). -->
<div class="min-h-dvh p-0 sm:p-4">
  <div class="grid min-h-dvh grid-cols-1 sm:min-h-[calc(100dvh-2rem)] sm:rounded-2xl sm:border lg:grid-cols-[minmax(0,0.85fr)_minmax(0,1fr)]">
    <aside class="relative m-3 hidden overflow-hidden rounded-xl border bg-card lg:block">
      <AuthArt />
      <div class="relative flex h-full flex-col justify-between p-8">
        <div class="flex items-center gap-3">
          <span class="grid size-10 place-items-center rounded-lg border bg-background">
            <OxevuMark class="size-5" />
          </span>
          <span class="font-semibold tracking-tight">Oxevu</span>
        </div>
        <p class="text-[2rem] leading-[1.1] font-semibold tracking-tight">
          Your library,<br /><span class="text-muted-foreground">played as it is.</span>
        </p>
      </div>
    </aside>

    <section class="flex items-center justify-center px-5 py-12">
      <div class="animate-rise flex w-full max-w-sm flex-col gap-8">
        <div class="flex items-center gap-2 lg:hidden">
          <OxevuMark />
          <span class="font-semibold tracking-tight">Oxevu</span>
        </div>
        {#key mode}
          <AuthForm {mode} {notice} />
        {/key}
      </div>
    </section>
  </div>
</div>
