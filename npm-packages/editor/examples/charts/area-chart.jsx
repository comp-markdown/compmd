// A line of `{label, value}` rows in order, over a faint wash. A crosshair follows the pointer to the nearest label
// and shows its value; the last value is labeled at the line's end.
import { useEffect, useRef, useState } from "react";

const HEIGHT = 220;
const TOP = 12;
const BOTTOM = 28;
const LEFT = 48;
const RIGHT = 56;

/** The element's width, followed as it resizes. */
function useWidth() {
  const ref = useRef(null);
  const [width, setWidth] = useState(0);
  useEffect(() => {
    const element = ref.current;
    if (!element) return undefined;
    const observer = new ResizeObserver(([entry]) => setWidth(entry.contentRect.width));
    observer.observe(element);
    return () => observer.disconnect();
  }, []);
  return [ref, width];
}

/** A clean top for the axis and its four steps. */
function scale(max) {
  if (max <= 0) return { top: 1, step: 0.25 };
  const raw = max / 4;
  const power = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * power).find((s) => s >= raw);
  return { top: step * 4, step };
}

const compact = (n) => new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 }).format(n);
const full = (n, unit) => `${new Intl.NumberFormat().format(n)}${unit ? ` ${unit}` : ""}`;

export default function AreaChart({ rows = [], unit, compmd }) {
  const [ref, width] = useWidth();
  const [hover, setHover] = useState(null);
  const { top, step } = scale(Math.max(0, ...rows.map((row) => row.value)));
  const plot = HEIGHT - TOP - BOTTOM;
  const span = Math.max(0, width - LEFT - RIGHT);
  const x = (i) => LEFT + (rows.length > 1 ? (span * i) / (rows.length - 1) : span / 2);
  const y = (value) => TOP + plot * (1 - value / top);
  const points = rows.map((row, i) => `${x(i)},${y(row.value)}`);
  const line = points.length > 0 ? `M${points.join("L")}` : "";
  const area = points.length > 0 ? `${line}L${x(rows.length - 1)},${y(0)}L${x(0)},${y(0)}Z` : "";
  const last = rows.at(-1);
  const nearest = (clientX, element) => {
    const left = element.getBoundingClientRect().left;
    const at = rows.length > 1 ? Math.round(((clientX - left - LEFT) / span) * (rows.length - 1)) : 0;
    return Math.min(rows.length - 1, Math.max(0, at));
  };
  const ticks = Array.from({ length: 5 }, (_, i) => i * step);
  const shown = hover !== null ? rows[hover] : undefined;
  return (
    <div ref={ref} style={{ position: "relative", fontFamily: "system-ui, sans-serif", fontSize: 12 }}>
      <svg
        width={width}
        height={HEIGHT}
        role="img"
        aria-label={compmd.fallback.text}
        tabIndex={0}
        style={{ display: "block", overflow: "visible", outline: "none" }}
        onPointerMove={(event) => setHover(nearest(event.clientX, event.currentTarget))}
        onPointerLeave={() => setHover(null)}
        onFocus={() => setHover(rows.length - 1)}
        onBlur={() => setHover(null)}
        onKeyDown={(event) => {
          if (event.key === "ArrowLeft") setHover((i) => Math.max(0, (i ?? 0) - 1));
          if (event.key === "ArrowRight") setHover((i) => Math.min(rows.length - 1, (i ?? 0) + 1));
        }}
      >
        {ticks.map((tick) => (
          <g key={tick}>
            <line x1={LEFT} x2={width - RIGHT} y1={y(tick)} y2={y(tick)} stroke={tick === 0 ? "var(--viz-axis, #c3c2b7)" : "var(--viz-grid, #e1e0d9)"} />
            <text x={LEFT - 8} y={y(tick)} dy="0.32em" textAnchor="end" fill="var(--viz-muted, #898781)" style={{ fontVariantNumeric: "tabular-nums" }}>
              {compact(tick)}
            </text>
          </g>
        ))}
        {rows.map((row, i) => (
          <text key={row.label} x={x(i)} y={HEIGHT - BOTTOM + 16} textAnchor="middle" fill="var(--viz-muted, #898781)">
            {row.label}
          </text>
        ))}
        <path d={area} fill="var(--viz-series-1, #2a78d6)" opacity={0.1} />
        <path d={line} fill="none" stroke="var(--viz-series-1, #2a78d6)" strokeWidth={2} strokeLinejoin="round" strokeLinecap="round" />
        {last && (
          <text x={x(rows.length - 1) + 8} y={y(last.value)} dy="0.32em" fill="var(--viz-primary, #0b0b0b)">
            {compact(last.value)}
          </text>
        )}
        {shown && (
          <g pointerEvents="none">
            <line x1={x(hover)} x2={x(hover)} y1={TOP} y2={y(0)} stroke="var(--viz-axis, #c3c2b7)" />
            <circle cx={x(hover)} cy={y(shown.value)} r={4} fill="var(--viz-series-1, #2a78d6)" stroke="var(--viz-surface, #fcfcfb)" strokeWidth={2} />
          </g>
        )}
      </svg>
      {shown && (
        <div
          role="status"
          style={{
            position: "absolute",
            left: x(hover),
            top: TOP,
            transform: `translate(${hover > rows.length / 2 ? "calc(-100% - 8px)" : "8px"}, 0)`,
            padding: "4px 8px",
            borderRadius: 6,
            background: "var(--viz-surface, #fcfcfb)",
            color: "var(--viz-primary, #0b0b0b)",
            boxShadow: "0 0 0 1px var(--viz-border, rgba(11,11,11,0.1)), 0 2px 6px rgba(0,0,0,0.12)",
            whiteSpace: "nowrap",
            pointerEvents: "none",
          }}
        >
          <strong>{full(shown.value, unit)}</strong> <span style={{ color: "var(--viz-secondary, #52514e)" }}>{shown.label}</span>
        </div>
      )}
    </div>
  );
}
