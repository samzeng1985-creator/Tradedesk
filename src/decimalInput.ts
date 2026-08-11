export const MAX_MONEY_INPUT = 999_999_999_999.99;

export function normalizeMoneyInput(value: string): string | null {
  const normalized = value.replace(",", ".");
  return /^\d*(?:\.\d{0,2})?$/.test(normalized) ? normalized : null;
}

export function moneyInputToMinor(value: string): number | null {
  const normalized = value.trim();
  if (!normalized) return null;

  const amount = Number(normalized);
  if (!Number.isFinite(amount) || amount <= 0 || amount > MAX_MONEY_INPUT) return null;
  return Math.round(amount * 100);
}
