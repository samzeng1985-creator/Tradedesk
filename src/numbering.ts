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

export function nextPurchaseSplitNumber(
  caseRecord: { id: string; number: string },
  orders: Array<{ businessCaseId: string; number: string }>,
) {
  const base = `PO-${caseRecord.number.replace(/^TD-/i, "")}`;
  const maximum = orders.reduce((current, order) => {
    if (order.businessCaseId !== caseRecord.id || !order.number.startsWith(`${base}-`)) return current;
    const sequence = Number(order.number.slice(base.length + 1));
    return Number.isInteger(sequence) ? Math.max(current, sequence) : current;
  }, 0);
  return `${base}-${maximum + 1}`;
}
