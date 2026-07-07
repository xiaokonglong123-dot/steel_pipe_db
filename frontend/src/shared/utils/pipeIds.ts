/**
 * Parse comma/space/newline separated pipe IDs, supporting ranges (e.g. "1001-1010").
 * Returns deduplicated sorted numeric IDs.
 */
export function parsePipeIds(input: string): number[] {
  const tokens = input
    .replace(/[，、；;\r\n\t]+/g, ',')
    .replace(/\s+/g, ',')
    .split(',')
    .map((token) => token.trim())
    .filter(Boolean);

  const ids: number[] = [];
  for (const token of tokens) {
    const rangeMatch = token.match(/^(\d+)\s*-\s*(\d+)$/);
    if (rangeMatch) {
      const start = Number(rangeMatch[1]);
      const end = Number(rangeMatch[2]);
      if (Number.isInteger(start) && Number.isInteger(end) && start > 0 && end >= start) {
        for (let id = start; id <= end; id += 1) {
          ids.push(id);
        }
      }
      continue;
    }

    const id = Number(token);
    if (Number.isInteger(id) && id > 0) {
      ids.push(id);
    }
  }

  return [...new Set(ids)];
}
