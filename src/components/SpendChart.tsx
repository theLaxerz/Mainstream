import { formatMoney, type FinanceDaySpend } from "../lib/finance";

type Props = {
  days: FinanceDaySpend[];
  label?: string;
};

export function SpendChart({ days, label = "Spend · 14d" }: Props) {
  const width = 168;
  const height = 44;
  const padX = 2;
  const padY = 3;
  if (days.length === 0) return null;

  const max = Math.max(...days.map((d) => d.spent), 1);
  const gap = 2;
  const barW = Math.max(
    2,
    (width - padX * 2 - gap * (days.length - 1)) / days.length,
  );
  const total = days.reduce((sum, d) => sum + d.spent, 0);

  return (
    <div className="spend-chart" aria-label={`${label} ${formatMoney(total)}`}>
      <svg
        className="spend-chart-svg"
        viewBox={`0 0 ${width} ${height}`}
        width={width}
        height={height}
        aria-hidden="true"
      >
        {days.map((d, i) => {
          const h = Math.max(1.5, (d.spent / max) * (height - padY * 2));
          const x = padX + i * (barW + gap);
          const y = height - padY - h;
          return (
            <rect
              key={d.day}
              x={x}
              y={y}
              width={barW}
              height={h}
              rx={1.5}
              fill="var(--accent)"
              opacity={d.spent > 0 ? 0.92 : 0.22}
            >
              <title>
                {d.day}: {formatMoney(d.spent)}
              </title>
            </rect>
          );
        })}
      </svg>
      <div className="sparkline-copy">
        <p className="sparkline-label">{label}</p>
        <p className="sparkline-value">{formatMoney(total)}</p>
      </div>
    </div>
  );
}
