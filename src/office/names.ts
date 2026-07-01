// Coworker names are matched case-insensitively everywhere (mailboxes,
// statuses, scheduled wakes); this is the single normalization used for keys.
export function normalizeCoworkerName(name: string): string {
  return name.trim().toLowerCase()
}
