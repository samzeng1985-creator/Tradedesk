export function todayIso() {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, "0");
  const day = String(now.getDate()).padStart(2, "0");
  return `${year}-${month}-${day}`;
}

export function dateStamp(date = todayIso()) {
  return date.replace(/-/g, "");
}

export function nextDatedNumber(existing: string[], prefix: string, date = todayIso()) {
  const base = `${prefix}-${dateStamp(date)}-`;
  const maximum = existing.reduce((current, value) => {
    if (!value.startsWith(base)) return current;
    const sequence = Number(value.slice(base.length));
    return Number.isInteger(sequence) ? Math.max(current, sequence) : current;
  }, 0);
  return `${base}${String(maximum + 1).padStart(4, "0")}`;
}
