import { factLabels, type FactId } from "$lib/media/file-facts";

let ids = $state<FactId[]>([]);
let account: string | undefined;

export const factPins = {
  get ids() { return ids; },
  load(user: string | undefined) {
    if (account === user) return;
    account = user;
    let stored: unknown;
    try { stored = JSON.parse(localStorage.getItem(`oxevu.fact-pins.${user}`) ?? "[]"); } catch { stored = []; }
    ids = Array.isArray(stored) ? [...new Set(stored.filter((id): id is FactId => typeof id === "string" && Object.hasOwn(factLabels, id)))] : [];
  },
  toggle(id: FactId) {
    ids = ids.includes(id) ? ids.filter((v) => v !== id) : [...ids, id];
    try { localStorage.setItem(`oxevu.fact-pins.${account}`, JSON.stringify(ids)); } catch { /* Session-only preferences when browser storage is unavailable. */ }
  },
};
