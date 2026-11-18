<!-- A donut of `{label, value}` rows, each a color in row order, with the total in its middle and a legend that
     gives every label, value and share. Hovering or focusing a slice or its legend entry lifts it. -->
<script>
  let { rows = [], unit, compmd } = $props();

  const SERIES = ["#2a78d6", "#eb6834", "#1baf7a", "#eda100", "#e87ba4", "#008300", "#6250d6", "#e34948"];
  const SIZE = 180;
  const RADIUS = 80;
  const THICK = 24;

  let hover = $state(null);

  const total = $derived(rows.reduce((sum, row) => sum + Math.max(0, row.value), 0));
  const slices = $derived.by(() => {
    let start = -Math.PI / 2;
    return rows.map((row, i) => {
      const angle = total > 0 ? (Math.max(0, row.value) / total) * Math.PI * 2 : 0;
      const slice = { row, i, start, end: start + angle };
      start += angle;
      return slice;
    });
  });

  const point = (angle, radius) => `${SIZE / 2 + radius * Math.cos(angle)},${SIZE / 2 + radius * Math.sin(angle)}`;

  /** A ring segment from `start` to `end`. */
  function arc(start, end) {
    const outer = RADIUS;
    const inner = RADIUS - THICK;
    // A whole ring is drawn as two halves, since an arc can't end where it starts.
    const finish = end - start >= Math.PI * 2 ? end - 0.0001 : end;
    const large = finish - start > Math.PI ? 1 : 0;
    return `M${point(start, outer)}A${outer},${outer} 0 ${large} 1 ${point(finish, outer)}L${point(finish, inner)}A${inner},${inner} 0 ${large} 0 ${point(start, inner)}Z`;
  }

  const color = (i) => `var(--viz-series-${(i % SERIES.length) + 1}, ${SERIES[i % SERIES.length]})`;
  const compact = (n) => new Intl.NumberFormat(undefined, { notation: "compact", maximumFractionDigits: 1 }).format(n);
  const full = (n) => `${new Intl.NumberFormat().format(n)}${unit ? ` ${unit}` : ""}`;
  const share = (n) => (total > 0 ? `${Math.round((n / total) * 100)}%` : "–");
</script>

<div class="pie">
  <svg width={SIZE} height={SIZE} viewBox="0 0 {SIZE} {SIZE}" role="img" aria-label={compmd.fallback.text}>
    {#each slices as { row, i, start, end } (row.label)}
      <path
        d={arc(start, end)}
        fill={color(i)}
        class:faded={hover !== null && hover !== i}
        role="presentation"
        onpointerenter={() => (hover = i)}
        onpointerleave={() => (hover = null)}
      />
    {/each}
    <text x={SIZE / 2} y={SIZE / 2} class="total">{compact(hover !== null ? rows[hover].value : total)}</text>
    <text x={SIZE / 2} y={SIZE / 2 + 18} class="caption">{hover !== null ? rows[hover].label : "total"}</text>
  </svg>
  <ul class="legend">
    {#each rows as row, i (row.label)}
      <li>
        <button type="button" class:faded={hover !== null && hover !== i} onpointerenter={() => (hover = i)} onpointerleave={() => (hover = null)} onfocus={() => (hover = i)} onblur={() => (hover = null)}>
          <span class="swatch" style:background={color(i)}></span>
          <span class="label">{row.label}</span>
          <strong>{full(row.value)}</strong>
          <span class="share">{share(row.value)}</span>
        </button>
      </li>
    {/each}
  </ul>
</div>

<style>
  .pie {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    font: 13px system-ui, sans-serif;
  }
  path {
    stroke: var(--viz-surface, #fcfcfb);
    stroke-width: 2px;
    transition: opacity 120ms;
  }
  .faded {
    opacity: 0.4;
  }
  .total {
    text-anchor: middle;
    font-size: 20px;
    font-weight: 600;
    fill: var(--viz-primary, #0b0b0b);
  }
  .caption {
    text-anchor: middle;
    fill: var(--viz-muted, #898781);
  }
  .legend {
    list-style: none;
    margin: 0;
    padding: 0;
    display: grid;
    gap: 6px;
  }
  button {
    display: grid;
    grid-template-columns: 10px auto auto auto;
    align-items: center;
    gap: 8px;
    border: 0;
    padding: 0;
    background: none;
    color: var(--viz-primary, #0b0b0b);
    font: inherit;
    text-align: start;
    cursor: default;
  }
  .swatch {
    width: 10px;
    height: 10px;
    border-radius: 2px;
  }
  .label,
  .share {
    color: var(--viz-secondary, #52514e);
  }
  strong,
  .share {
    font-variant-numeric: tabular-nums;
  }
</style>
