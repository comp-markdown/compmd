---
md:
  version: 1
  import:
    - from: ./harvest.jsx
      names: [harvest]
title: Elm Street Garden
issue: November 2026
harvest:
  - {crop: Beans, kilos: 42}
  - {crop: Leeks, kilos: 25}
  - {crop: Kale, kilos: 18}
plots:
  - {number: 1, gardener: Maya Okafor}
  - {number: 2, gardener: Tomás Reyes}
  - {number: 3, gardener: ""}
  - {number: 4, gardener: Priya Nair}
  - {number: 5, gardener: ""}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## This year's harvest

:: harvest[Beans 42 kg, leeks 25 kg, kale 18 kg] {rows=(md.doc.harvest) unit=kg}

## The plots

:: plot-map[Plots 1, 2 and 4 are taken; plots 3 and 5 are free] {plots=(md.doc.plots)}

```compmd component=harvest signature
params:
  rows: array
  unit: string = kg
```

```svelte component=plot-map runtime=svelte@5
<script>
  let { plots, compmd } = $props();
</script>

<div role="img" aria-label={compmd.fallback.text} style="display: flex; gap: 0.5rem">
  {#each plots as plot}
    <span style="padding: 0.5rem 0.75rem; border: 1px solid; opacity: {plot.gardener ? 1 : 0.4}">{plot.number}</span>
  {/each}
</div>
```

```compmd component=plot-map signature
params:
  plots: array
```
