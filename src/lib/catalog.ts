import type { VaultEntry } from "../types/vault";

export function filterEntries(
  entries: VaultEntry[],
  query: string,
  groupId?: string,
  favoritesOnly = false,
): VaultEntry[] {
  const normalized = query.trim().toLocaleLowerCase();
  return entries.filter((entry) => {
    if (groupId && entry.groupId !== groupId) return false;
    if (favoritesOnly && !entry.favorite) return false;
    if (!normalized) return true;
    const haystack = [
      entry.title,
      entry.username,
      entry.url,
      entry.notes,
      ...entry.tags,
      ...entry.customFields.flatMap((field) => [field.key, field.value]),
    ]
      .join(" ")
      .toLocaleLowerCase();
    return haystack.includes(normalized);
  });
}
