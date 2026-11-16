import { onDestroy, onMount, untrack } from "svelte";
import { errorMessage } from "$lib/api/client";
import { Autosave } from "./autosave";
import { pick, type SettingsDocument } from "./document";
import { createResource, type ResourceStatus } from "./resource.svelte";

export type FieldErrors<T> = Partial<Record<keyof T, string>>;
export type SavePhase = "idle" | "pending" | "saving" | "saved" | "error";

/**
 * Autosaving draft for the fields one settings page owns. The draft holds the
 * whole record so field components keep their types, but only `keys` are
 * validated, compared and written; other pages' fields are never sent from
 * here. Call during component initialisation.
 */
export function createSettingsForm<E, T extends object, K extends keyof T>(
  document: SettingsDocument<E, T>,
  keys: readonly K[],
  validate: (draft: T, envelope: E) => FieldErrors<T>,
) {
  const resource = createResource(() => document.load());
  let draft = $state<T | null>(null);
  let phase = $state<SavePhase>("idle");
  let failure = $state<string | null>(null);

  const errors = $derived.by(() => {
    if (!draft || !resource.value) return {};
    const all = validate(draft, resource.value);
    const own: FieldErrors<T> = {};
    for (const key of keys) if (all[key]) own[key] = all[key];
    return own;
  });
  const invalid = $derived(Object.keys(errors).length > 0);
  const dirty = $derived.by(() => {
    if (!draft || !resource.value) return false;
    const saved = document.settingsOf(resource.value);
    return keys.some((key) => draft?.[key] !== saved[key]);
  });

  const autosave = new Autosave<Pick<T, K>>(
    async (value) => {
      resource.value = await document.patch(value as Partial<T>);
    },
    (state, error) => {
      phase = state;
      failure = state === "error" ? errorMessage(error) : null;
    },
  );

  async function load() {
    await resource.load();
    if (resource.value && !draft) {
      // The resource value is a reactive proxy; snapshot it before copying.
      const settings = $state.snapshot(document.settingsOf(resource.value)) as T;
      autosave.reset(pick(settings, keys));
      draft = settings;
    }
  }

  onMount(() => {
    void load();
  });
  $effect(() => {
    const value = draft && !invalid ? pick($state.snapshot(draft) as T, keys) : null;
    untrack(() => {
      autosave.update(value);
      if (autosave.queued) phase = "pending";
    });
  });
  // Leaving the page sends a pending edit right away instead of dropping it.
  onDestroy(() => {
    void autosave.flush();
  });

  return {
    get draft() {
      return draft;
    },
    set draft(value: T | null) {
      draft = value;
    },
    get envelope() {
      return resource.value;
    },
    get status() {
      return resource.status;
    },
    get loadError() {
      return resource.error;
    },
    /** Errors for owned fields, shown once something was edited. */
    get errors(): FieldErrors<T> {
      return dirty ? errors : {};
    },
    get invalid() {
      return invalid;
    },
    get dirty() {
      return dirty;
    },
    get phase() {
      return phase;
    },
    get failure() {
      return failure;
    },
    load,
  };
}

/** What the shared form body needs, independent of the record type. */
export interface SettingsFormView {
  readonly draft: unknown;
  readonly status: ResourceStatus;
  readonly loadError: string | null;
  readonly invalid: boolean;
  readonly phase: SavePhase;
  readonly failure: string | null;
  load: () => Promise<void>;
}
