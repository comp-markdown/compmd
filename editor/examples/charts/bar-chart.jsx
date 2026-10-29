// Columns of `{label, value}` rows from one baseline. When `selected` is bound to an input, clicking a column (or
// Enter on it) sets the input to its label, and the other columns recede.
import { useEffect, useRef, useState } from "react";

const HEIGHT = 220;
const TOP = 12;
const BOTTOM = 28;
const LEFT = 48;
const BAR = 24;

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

/** A clean top for the axis and its four steps: 0, 2.5K, 5K, 7.5K, 10K. */
function scale(max) {
  if (max <= 0) return { top: 1, step: 0.25 };
  const raw = max / 4;
  const power = 10 ** Math.floor(Math.log10(raw));
  const step = [1, 2, 2.5, 5, 10].map((m) => m * power).find((s) => s >= raw);
  return { top: step * 4, step };
}

const compact = (n) => new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 }).format(n);
const full = (n, unit) => `${new Intl.NumberFormat().format(n)}${unit ? ` ${unit}` : ""}`;

/** A column with a 4px rounded data end and a square foot on the baseline. */
function column(x, y, width, height) {
  const r = Math.min(4, height, width / 2);
  return `M${x},${y + height}V${y + r}Q${x},${y} ${x + r},${y}H${x + width - r}Q${x + width},${y} ${x + width},${y + r}V${y + height}Z`;
}

export default function BarChart({ rows = [], unit, selected, compmd }) {
  const [ref, width] = useWidth();
  const [hover, setHover] = useState(null);
  const { top, step } = scale(Math.max(0, ...rows.map((row) => row.value)));
  const plot = HEIGHT - TOP - BOTTOM;
  const y = (value) => TOP + plot * (1 - value / top);
  const band = rows.length > 0 ? Math.max(0, width - LEFT) / rows.length : 0;
  const thick = Math.min(BAR, band * 0.6);
  const choosing = selected !== undefined;
  const chosen = rows.some((row) => row.label === selected) ? selected : undefined;
  const choose = (row) => {
    if (choosing) compmd.set("selected", row.label);
  };
  const ticks = Array.from({ length: 5 }, (_, i) => i * step);
  return (
    <div ref={ref} style={{ position: "relative", fontFamily: "system-ui, sans-serif", fontSize: 12 }}>
      <svg width={width} height={HEIGHT} role="img" aria-label={compmd.fallback.text} style={{ display: "block", overflow: "visible" }}>
        {ticks.map((tick) => (
          <g key={tick}>
            <line x1={LEFT} x2={width} y1={y(tick)} y2={y(tick)} stroke={tick === 0 ? "var(--viz-axis, #c3c2b7)" : "var(--viz-grid, #e1e0d9)"} />
            <text x={LEFT - 8} y={y(tick)} dy="0.32em" textAnchor="end" fill="var(--viz-muted, #898781)" style={{ fontVariantNumeric: "tabular-nums" }}>
              {compact(tick)}
            </text>
          </g>
        ))}
        {rows.map((row, i) => {
          const x = LEFT + band * i + (band - thick) / 2;
          const height = Math.max(0, y(0) - y(row.value));
          const recedes = chosen !== undefined && row.label !== chosen;
          return (
            <g
              key={row.label}
              tabIndex={0}
              role={choosing ? "button" : undefined}
              aria-pressed={choosing ? row.label === chosen : undefined}
              aria-label={`${row.label}: ${full(row.value, unit)}`}
              style={{ cursor: choosing ? "pointer" : "default", outline: "none" }}
              onPointerEnter={() => setHover(i)}
              onPointerLeave={() => setHover(null)}
              onFocus={() => setHover(i)}
              onBlur={() => setHover(null)}
              onClick={() => choose(row)}
              onKeyDown={(event) => {
                if (event.key === "Enter" || event.key === " ") {
                  event.preventDefault();
                  choose(row);
                }
              }}
            >
              <rect x={LEFT + band * i} y={TOP} width={band} height={plot} fill="transparent" />
              <path d={column(x, y(row.value), thick, height)} fill="var(--viz-series-1, #2a78d6)" opacity={recedes ? 0.35 : hover === i ? 0.85 : 1} />
              <text x={x + thick / 2} y={HEIGHT - BOTTOM + 16} textAnchor="middle" fill="var(--viz-secondary, #52514e)">
                {row.label}
              </text>
            </g>
          );
        })}
      </svg>
      {hover !== null && rows[hover] && (
        <div
          role="status"
          style={{
            position: "absolute",
            left: LEFT + band * hover + band / 2,
            top: y(rows[hover].value) - 8,
            transform: "translate(-50%, -100%)",
            padding: "4px 8px",
            borderRadius: 6,
            background: "var(--viz-surface, #fcfcfb)",
            color: "var(--viz-primary, #0b0b0b)",
            boxShadow: "0 0 0 1px var(--viz-border, rgba(11,11,11,0.1)), 0 2px 6px rgba(0,0,0,0.12)",
            whiteSpace: "nowrap",
            pointerEvents: "none",
          }}
        >
          <strong>{full(rows[hover].value, unit)}</strong> <span style={{ color: "var(--viz-secondary, #52514e)" }}>{rows[hover].label}</span>
        </div>
      )}
    </div>
  );
}
