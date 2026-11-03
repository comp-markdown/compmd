---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic}
  - {number: 2, gardener: Tomás Reyes, crop: kale}
  - {number: 3, gardener: "", crop: ""}
  - {number: 4, gardener: Priya Nair, crop: leeks}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

```compmd component=plot-card {padding=s border=subtle border-radius=m background=(gardener == "" ? "accent" : "none")}
params:
  number: number
  gardener: string
  crop: string
---
# Plot {{number}}

:value[Maya Okafor, growing garlic.]{of=(gardener == "" ? "Free for next year." : gardener + ", growing " + crop + ".")}
```

```compmd component=gardener
params:
  name: string
  plot: number
---
**{{name}}** (plot {{plot}})
```

## The plots

::: grid {min=12rem gap=m}

::: each {in=(md.doc.plots) as=p}

:: plot-card[Plot {{p.number}}] {number=(p.number) gardener=(p.gardener) crop=(p.crop)}

::: /each

::: /grid

## Thanks

To :gardener[Maya Okafor (plot 1)]{name="Maya Okafor" plot=1} for turning the compost, and to :gardener[Priya Nair (plot 4)]{name="Priya Nair" plot=4} for mending the gate.
