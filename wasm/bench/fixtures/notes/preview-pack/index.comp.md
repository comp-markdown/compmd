---
md:
  version: 1
  export:
    names: "*"
---

The preview's chart components, as an ambient pack the frame implements (spec §6.3): their signatures here, their code in `components/`.

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

```compmd component=pie-chart signature
params:
  rows: array
  unit: string?
```

```compmd component=timeline signature
params:
  events: array
  kinds: array?
```

```compmd component=data-table signature
params:
  rows: array
  cols: array
  unit: string?
```
