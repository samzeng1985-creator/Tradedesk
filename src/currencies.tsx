export const COMMON_CURRENCIES = [
  ["CNY", "人民币"],
  ["USD", "美元"],
  ["EUR", "欧元"],
  ["RUB", "俄罗斯卢布"],
  ["AUD", "澳大利亚元"],
  ["GBP", "英镑"],
  ["CAD", "加拿大元"],
  ["JPY", "日元"],
  ["CHF", "瑞士法郎"],
  ["AED", "阿联酋迪拉姆"],
  ["SAR", "沙特里亚尔"],
  ["SGD", "新加坡元"],
  ["HKD", "港元"],
  ["KRW", "韩元"],
  ["INR", "印度卢比"],
  ["BRL", "巴西雷亚尔"],
  ["MXN", "墨西哥比索"],
  ["ZAR", "南非兰特"],
  ["TRY", "土耳其里拉"],
  ["MYR", "马来西亚林吉特"],
  ["THB", "泰铢"],
  ["IDR", "印度尼西亚盾"],
  ["VND", "越南盾"],
] as const;

export function CurrencySelect({
  value,
  onChange,
  disabled = false,
  ariaLabel,
}: {
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
  ariaLabel?: string;
}) {
  const known = COMMON_CURRENCIES.some(([code]) => code === value);
  return (
    <select required value={value} disabled={disabled} aria-label={ariaLabel} onChange={(event) => onChange(event.target.value)}>
      {!known && value && <option value={value}>{value} · 已有币种</option>}
      {COMMON_CURRENCIES.map(([code, name]) => <option value={code} key={code}>{code} · {name}</option>)}
    </select>
  );
}

export function formatMoney(valueMinor: number, currency: string) {
  try {
    return new Intl.NumberFormat("zh-CN", {
      style: "currency",
      currency: currency || "CNY",
      maximumFractionDigits: 2,
    }).format(valueMinor / 100);
  } catch {
    return `${currency || "CNY"} ${(valueMinor / 100).toFixed(2)}`;
  }
}
