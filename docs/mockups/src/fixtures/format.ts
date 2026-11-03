// Number and money formats shared by every screen.

const whole = new Intl.NumberFormat("en-US", { maximumFractionDigits: 0 });
const cents = new Intl.NumberFormat("en-US", {
  minimumFractionDigits: 2,
  maximumFractionDigits: 2,
});

/** 17772 -> "17,772". */
export function formatCount(value: number): string {
  return whole.format(value);
}

/** 49 -> "$49", 309.2 -> "$309.20". Whole amounts drop the cents. */
export function formatMoney(value: number): string {
  return Number.isInteger(value)
    ? `$${whole.format(value)}`
    : `$${cents.format(value)}`;
}

/** Always with cents: 49 -> "$49.00". For totals in tables. */
export function formatMoneyCents(value: number): string {
  return `$${cents.format(value)}`;
}

/** 0.4356 -> "44%". */
export function formatPercent(fraction: number): string {
  return `${Math.round(fraction * 100)}%`;
}
