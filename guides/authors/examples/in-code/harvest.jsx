export default function Harvest({ rows, unit, compmd }) {
  const most = Math.max(...rows.map((row) => row.kilos));
  return (
    <figure role="img" aria-label={compmd.fallback.text} style={{ margin: 0 }}>
      {rows.map((row) => (
        <div key={row.crop} style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
          <span style={{ width: "4rem" }}>{row.crop}</span>
          <span style={{ width: `${(row.kilos / most) * 60}%`, height: "0.75rem", background: "currentColor" }} />
          <span>
            {row.kilos} {unit}
          </span>
        </div>
      ))}
    </figure>
  );
}
