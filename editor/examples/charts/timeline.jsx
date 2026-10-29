// Events `{date, title, kind}` in date order down a line, each kind a color in the order `kinds` gives (else first
// seen), with a legend that names them.

const SERIES = [
  ["--viz-series-1", "#2a78d6"],
  ["--viz-series-2", "#eb6834"],
  ["--viz-series-3", "#1baf7a"],
];

const color = (index) => {
  const [name, fallback] = SERIES[index % SERIES.length];
  return `var(${name}, ${fallback})`;
};

const day = (date) => new Intl.DateTimeFormat(undefined, { month: "short", day: "numeric", timeZone: "UTC" }).format(new Date(`${date}T00:00:00Z`));

export default function Timeline({ events = [], kinds, compmd }) {
  const sorted = [...events].sort((a, b) => (a.date < b.date ? -1 : a.date > b.date ? 1 : 0));
  const order = kinds ?? [...new Set(sorted.map((event) => event.kind).filter(Boolean))];
  const swatch = (kind) => color(Math.max(0, order.indexOf(kind)));
  return (
    <div style={{ fontFamily: "system-ui, sans-serif", fontSize: 13 }}>
      {order.length > 1 && (
        <ul aria-label="Kinds" style={{ display: "flex", flexWrap: "wrap", gap: 16, listStyle: "none", margin: "0 0 12px", padding: 0, color: "var(--viz-secondary, #52514e)" }}>
          {order.map((kind) => (
            <li key={kind} style={{ display: "flex", alignItems: "center", gap: 6 }}>
              <span aria-hidden="true" style={{ width: 10, height: 10, borderRadius: 2, background: swatch(kind) }} />
              {kind}
            </li>
          ))}
        </ul>
      )}
      <ol aria-label={compmd.fallback.text} style={{ listStyle: "none", margin: 0, padding: 0, borderInlineStart: "1px solid var(--viz-axis, #c3c2b7)" }}>
        {sorted.map((event) => (
          <li key={`${event.date} ${event.title}`} style={{ position: "relative", padding: "0 0 12px 20px" }}>
            <span
              aria-hidden="true"
              style={{ position: "absolute", insetInlineStart: -6, top: 3, width: 8, height: 8, borderRadius: "50%", background: swatch(event.kind), boxShadow: "0 0 0 2px var(--viz-surface, #fcfcfb)" }}
            />
            <time dateTime={event.date} style={{ color: "var(--viz-muted, #898781)", fontVariantNumeric: "tabular-nums", marginInlineEnd: 8 }}>
              {day(event.date)}
            </time>
            <span style={{ color: "var(--viz-primary, #0b0b0b)" }}>{event.title}</span>
            {event.kind && <span style={{ color: "var(--viz-secondary, #52514e)" }}> · {event.kind}</span>}
          </li>
        ))}
      </ol>
    </div>
  );
}
