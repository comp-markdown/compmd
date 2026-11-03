---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
beds: [North, South, East]
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic, bed: North}
  - {number: 2, gardener: Tomás Reyes, crop: kale, bed: North}
  - {number: 3, gardener: Jun Park, crop: beans, bed: South}
  - {number: 4, gardener: Priya Nair, crop: leeks, bed: South}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

```compmd component=bed {display=flex flex-direction=column gap=s padding=m border=subtle border-radius=m}
params:
  title: string
  plots: array
  as: name
slots:
  default: {each: plots, as: as}
  empty: {}
---
# The {{title}} bed

:value[2 plots]{of=(string(size(plots)) + " " + md.plural(size(plots), "plot", "plots"))}

{{md.slot}}

::: if {when=(size(plots) == 0)}

{{md.slot.empty}}

::: /if
```

## The beds

::: grid {min=14rem gap=m}

::: each {in=(md.doc.beds) as=b}

::: bed {title="{{b}}" plots=(md.doc.plots.filter(p, p.bed == b)) as=plot}

**Plot {{plot.number}}**: {{plot.gardener}}, growing {{plot.crop}}.

:: #empty

No plots here yet. Ask Maya for one.

::: /bed

::: /each

::: /grid
