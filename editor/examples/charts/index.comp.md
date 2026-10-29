---
md:
  version: 1
  import:
    - from: ./bar-chart.jsx
      names: [bar-chart]
    - from: ./area-chart.jsx
      names: [area-chart]
    - from: ./timeline.jsx
      names: [timeline]
    - from: ./pie-chart.svelte
      names: [pie-chart]
    - from: ./data-table.svelte
      names: [data-table]
  export:
    names: [bar-chart, area-chart, timeline, pie-chart, data-table]
---

Charts for the editor's examples, in plain SVG: React components for bars, areas and a timeline, Svelte components
for a donut and a sortable table. Rows are `{label, value}` records unless a signature says otherwise. Colors come
from `--viz-*` custom properties when the page sets them.

```compmd component=bar-chart signature
params:
  rows: array
  unit: string?
  selected: input<enum>?
```

```compmd component=area-chart signature
params:
  rows: array
  unit: string?
```

```compmd component=timeline signature
params:
  events: array
  kinds: array?
```

```compmd component=pie-chart signature
params:
  rows: array
  unit: string?
```

```compmd component=data-table signature
params:
  rows: array
  cols: array
  unit: string?
```
