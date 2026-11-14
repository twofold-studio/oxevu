// Readable URL segments for series. A show's internal key stays
// `{library id}\0{title}`; the slug only exists for links, so it can change
// without touching progress or grouping.

const FALLBACK = "series";
const MAX_LENGTH = 80;

/**
 * Lowercase letters and digits from any script, joined by single dashes.
 * Accents are folded only on Latin letters ("Pokémon" → "pokemon"); other
 * scripts keep their combining marks because there they change the letter
 * (Devanagari vowel signs, Japanese dakuten).
 */
export function slugify(title: string): string {
  const slug = title
    .normalize("NFKD")
    .replace(/(\p{Script=Latin})\p{M}+/gu, "$1")
    .normalize("NFC")
    .toLowerCase()
    // "Grey's" reads better as "greys" than "grey-s".
    .replace(/['’]/g, "")
    .replace(/[^\p{L}\p{N}\p{M}]+/gu, "-")
    .replace(/^-+|-+$/g, "");
  // Cut on code points so a surrogate pair is never split.
  const bounded = Array.from(slug).slice(0, MAX_LENGTH).join("").replace(/-+$/, "");
  return bounded || FALLBACK;
}

// Plain code-unit order: locale collation could differ between browsers and
// reshuffle which show keeps the bare slug.
const byKey = (a: { key: string }, b: { key: string }) =>
  a.key < b.key ? -1 : a.key > b.key ? 1 : 0;

/**
 * Assigns every show a unique slug. Shows whose titles slugify the same
 * (one title in two libraries, or "Arcane!" next to "Arcane") are ordered by
 * key; the first keeps the bare slug and the rest get `-2`, `-3`, … skipping
 * any suffix that is another title's own slug ("Arcane 2" → "arcane-2").
 */
export function assignShowSlugs(
  shows: readonly { key: string; title: string }[],
): Map<string, string> {
  const groups = new Map<string, { key: string }[]>();
  for (const show of shows) {
    const base = slugify(show.title);
    const group = groups.get(base);
    if (group) group.push(show);
    else groups.set(base, [show]);
  }

  // Every bare slug is reserved up front so suffixes never steal one.
  const taken = new Set(groups.keys());
  const slugs = new Map<string, string>();
  for (const base of [...groups.keys()].sort()) {
    const [first, ...rest] = (groups.get(base) ?? []).toSorted(byKey);
    slugs.set(first.key, base);
    let n = 2;
    for (const show of rest) {
      while (taken.has(`${base}-${n}`)) n += 1;
      taken.add(`${base}-${n}`);
      slugs.set(show.key, `${base}-${n}`);
    }
  }
  return slugs;
}

/**
 * Finds a show from a URL segment: its slug, a legacy internal key from links
 * made before slugs existed, or a hand-typed title ("Arcane").
 */
export function findShow<T extends { key: string; slug: string }>(
  shows: readonly T[],
  ref: string,
): T | undefined {
  return (
    shows.find((show) => show.slug === ref) ??
    shows.find((show) => show.key === ref) ??
    shows.find((show) => show.slug === slugify(ref))
  );
}
