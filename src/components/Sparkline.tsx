type Props = {
  values: number[];
  label: string;
  formatValue?: (n: number) => string;
  accent?: string;
};

export function Sparkline({
  values,
  label,
  formatValue,
  accent = "var(--teal)",
}: Props) {
  const width = 120;
  const height = 36;
  const pad = 2;
  if (values.length === 0) return null;

  const min = Math.min(...values);
  const max = Math.max(...values);
  const span = max - min || 1;
  const step = values.length > 1 ? (width - pad * 2) / (values.length - 1) : 0;
  const points = values
    .map((v, i) => {
      const x = pad + i * step;
      const y = height - pad - ((v - min) / span) * (height - pad * 2);
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
  const latest = values[values.length - 1];
  const latestLabel = formatValue ? formatValue(latest) : String(latest);

  return (
    <div className="sparkline" aria-label={`${label} ${latestLabel}`}>
      <svg
        className="sparkline-svg"
        viewBox={`0 0 ${width} ${height}`}
        width={width}
        height={height}
        aria-hidden="true"
      >
        <polyline
          fill="none"
          stroke={accent}
          strokeWidth="2.2"
          strokeLinecap="round"
          strokeLinejoin="round"
          points={points}
        />
      </svg>
      <div className="sparkline-copy">
        <p className="sparkline-label">{label}</p>
        <p className="sparkline-value">{latestLabel}</p>
      </div>
    </div>
  );
}
