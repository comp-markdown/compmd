<!-- A table of `rows`, with the columns `cols` names: each a field name, or `{key, label}`. A column's header sorts
     the rows by it; a second press reverses the order. Numbers align at the end. -->
<script>
  let { rows = [], cols = [], unit, compmd } = $props();

  let sortKey = $state(null);
  let descending = $state(false);

  const columns = $derived(cols.map((col) => (typeof col === "string" ? { key: col, label: col } : col)));
  const numeric = (key) => rows.length > 0 && rows.every((row) => typeof row[key] === "number");
  const sorted = $derived.by(() => {
    if (sortKey === null) return rows;
    const key = sortKey;
    const order = [...rows].sort((a, b) => (a[key] < b[key] ? -1 : a[key] > b[key] ? 1 : 0));
    return descending ? order.reverse() : order;
  });

  function sortBy(key) {
    descending = sortKey === key ? !descending : false;
    sortKey = key;
  }

  const shown = (value) => (typeof value === "number" ? new Intl.NumberFormat().format(value) : String(value ?? ""));
</script>

<div class="scroll">
  <table aria-label={compmd.fallback.text}>
    <thead>
      <tr>
        {#each columns as col (col.key)}
          <th scope="col" class:number={numeric(col.key)} aria-sort={sortKey === col.key ? (descending ? "descending" : "ascending") : "none"}>
            <button type="button" onclick={() => sortBy(col.key)}>
              {col.label}{#if sortKey === col.key}<span aria-hidden="true">{descending ? " ↓" : " ↑"}</span>{/if}
            </button>
          </th>
        {/each}
      </tr>
    </thead>
    <tbody>
      {#each sorted as row, i (i)}
        <tr>
          {#each columns as col (col.key)}
            <td class:number={numeric(col.key)}>{shown(row[col.key])}</td>
          {/each}
        </tr>
      {/each}
    </tbody>
  </table>
  {#if unit}<p class="unit">Amounts in {unit}.</p>{/if}
</div>

<style>
  .scroll {
    overflow-x: auto;
    font: 13px system-ui, sans-serif;
  }
  table {
    border-collapse: collapse;
    width: 100%;
  }
  th,
  td {
    padding: 6px 10px;
    text-align: start;
    border-bottom: 1px solid var(--viz-grid, #e1e0d9);
    color: var(--viz-primary, #0b0b0b);
  }
  .number {
    text-align: end;
    font-variant-numeric: tabular-nums;
  }
  th button {
    border: 0;
    padding: 0;
    background: none;
    font: inherit;
    font-weight: 600;
    color: var(--viz-secondary, #52514e);
    cursor: pointer;
  }
  .unit {
    margin: 6px 0 0;
    color: var(--viz-muted, #898781);
  }
</style>
