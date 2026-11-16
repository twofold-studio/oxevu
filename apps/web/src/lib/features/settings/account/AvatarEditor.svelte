<script lang="ts">
  import * as Dialog from "$lib/components/ui/dialog";
  import { Button } from "$lib/components/ui/button";
  import { request, errorMessage } from "$lib/api/client";
  import type { User } from "$lib/api/types";
  import { session } from "$lib/state/session.svelte";
  import { categories, preset } from "./avatar-presets";

  let open = $state(false);
  let image = $state<HTMLImageElement | null>(null);
  let zoom = $state(1);
  let x = $state(0);
  let y = $state(0);
  let busy = $state(false);
  let failure = $state("");
  let category = $state<keyof typeof categories>("Abstract");
  let drag: { x: number; y: number } | null = null;
  const scale = $derived(image ? Math.max(256 / image.width, 256 / image.height) * zoom : 1);

  function clamp() {
    if (!image) return;
    const maxX = Math.max(0, (image.width * scale - 256) / 2);
    const maxY = Math.max(0, (image.height * scale - 256) / 2);
    x = Math.max(-maxX, Math.min(maxX, x));
    y = Math.max(-maxY, Math.min(maxY, y));
  }

  async function upload(event: Event) {
    const file = (event.currentTarget as HTMLInputElement).files?.[0];
    if (!file) return;
    failure = "";
    if (file.size > 10 * 1024 * 1024 || !["image/png", "image/jpeg", "image/webp"].includes(file.type)) {
      failure = "Choose a PNG, JPEG or WebP under 10 MB."; return;
    }
    try {
      const url = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => resolve(String(reader.result));
        reader.onerror = () => reject(new Error("Could not read image"));
        reader.readAsDataURL(file);
      });
      const loaded = new Image();
      loaded.src = url;
      await loaded.decode();
      image = loaded; zoom = 1; x = 0; y = 0;
    } catch { failure = "This image could not be opened."; }
  }

  async function choose(symbol: string, index: number) {
    const loaded = new Image();
    loaded.src = preset(symbol, index);
    await loaded.decode();
    image = loaded; zoom = 1; x = 0; y = 0;
  }

  async function save() {
    if (!image) return;
    busy = true; failure = "";
    try {
      clamp();
      const canvas = document.createElement("canvas");
      canvas.width = canvas.height = 256;
      const context = canvas.getContext("2d");
      if (!context) throw new Error("Image editing is unavailable");
      context.drawImage(image, 128 + x - image.width * scale / 2, 128 + y - image.height * scale / 2, image.width * scale, image.height * scale);
      const result = await request<{ user: User }>("/account/avatar", { method: "PUT", body: { image: canvas.toDataURL("image/png") } });
      session.user = result.user;
      open = false;
    } catch (error) { failure = errorMessage(error); }
    finally { busy = false; }
  }
</script>

<div class="flex items-center gap-4">
  {#if session.user?.avatar}<img src={session.user.avatar} alt="Your avatar" class="size-16 rounded-full border" />{/if}
  <Button variant="outline" onclick={() => { failure = ""; open = true; }}>Change avatar</Button>
</div>
<Dialog.Root bind:open>
  <Dialog.Content>
    <Dialog.Header><Dialog.Title>Your avatar</Dialog.Title><Dialog.Description>Upload a picture or choose a preset. Drag to position, then adjust the zoom.</Dialog.Description></Dialog.Header>
    <div class="mx-auto flex flex-col items-center gap-4">
      <div role="img" aria-label="Circular avatar crop preview" class="relative size-64 touch-none overflow-hidden rounded-full border bg-muted"
        onpointerdown={(event) => { event.currentTarget.setPointerCapture(event.pointerId); drag = { x: event.clientX, y: event.clientY }; }}
        onpointermove={(event) => { if (!drag || busy) return; x += event.clientX - drag.x; y += event.clientY - drag.y; drag = { x: event.clientX, y: event.clientY }; clamp(); }}
        onpointerup={() => { drag = null; }} onpointercancel={() => { drag = null; }}>
        {#if image}<img src={image.src} alt="" draggable="false" class="pointer-events-none absolute max-w-none" style:width={`${image.width * scale}px`} style:height={`${image.height * scale}px`} style:left={`${128 + x - image.width * scale / 2}px`} style:top={`${128 + y - image.height * scale / 2}px`} />{:else}<span class="grid h-full place-items-center text-muted-foreground">Choose an image</span>{/if}
      </div>
      <label class="flex w-full items-center gap-3 text-sm">Zoom<input aria-label="Avatar zoom" class="w-full" type="range" min="1" max="4" step="0.01" bind:value={zoom} oninput={() => { requestAnimationFrame(clamp); }} disabled={!image || busy} /></label>
      <div class="flex gap-2"><Button variant="outline" disabled={!image || busy} onclick={() => { x -= 10; clamp(); }}>←</Button><Button variant="outline" disabled={!image || busy} onclick={() => { x += 10; clamp(); }}>→</Button><Button variant="outline" disabled={!image || busy} onclick={() => { y -= 10; clamp(); }}>↑</Button><Button variant="outline" disabled={!image || busy} onclick={() => { y += 10; clamp(); }}>↓</Button></div>
    </div>
    <label class="flex flex-col gap-2 text-sm">Upload picture<input type="file" accept="image/png,image/jpeg,image/webp" onchange={upload} disabled={busy} /></label>
    <label class="flex items-center gap-3 text-sm">Presets<select class="rounded-md border bg-background p-2" bind:value={category}>{#each Object.keys(categories) as name (name)}<option>{name}</option>{/each}</select></label>
    <div class="grid grid-cols-5 gap-2">{#each categories[category] as symbol, index (symbol)}<button class="grid aspect-square place-items-center rounded-full border bg-secondary text-3xl hover:bg-accent focus-visible:outline-2 focus-visible:outline-ring" aria-label={`${category} avatar ${index + 1}: ${symbol}`} disabled={busy} onclick={() => choose(symbol, index)}>{symbol}</button>{/each}</div>
    <Button variant="outline" disabled={busy} onclick={() => { const values = Object.values(categories).flat(); const index = Math.floor(Math.random() * values.length); void choose(values[index], index % 5); }}>Surprise me</Button>
    {#if failure}<p role="alert" class="text-sm text-destructive">{failure}</p>{/if}
    <Dialog.Footer><Button variant="outline" disabled={busy} onclick={() => { open = false; }}>Cancel</Button><Button disabled={!image || busy} onclick={save}>{busy ? "Saving…" : "Save avatar"}</Button></Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
