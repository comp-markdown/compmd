---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
editor: {name: Maya Okafor, phone: 555-0142}
workday: "2026-11-07"
water_off: "2026-11-15"
fees: {small: 25, large: 40}
plots: {small: 12, large: 8, taken: 17}
helpers: [Maya, Tomás, Priya, Jun]
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}, from {{md.doc.editor}}.

::: box {padding=m border-radius=m background=(md.daysBetween(md.today(), md.doc.water_off) <= 14 ? "accent" : "subtle")}

**The water goes off on :value[November 15, 2026]{of=(md.date(md.doc.water_off, "long"))}** for the winter. Empty your cans and hoses before then.

::: /box

## Work day

:value[Saturday, November 7, 2026]{of=(md.date(md.doc.workday, "full"))}, from nine to noon, with :value[Maya, Tomás, Priya, and Jun]{of=(md.list(md.doc.helpers))} leading the jobs.

## Plots for next year

| Size | Plots | Fee for the year |
| --- | ---: | ---: |
| Small | {{md.doc.plots.small}} | :value[$25.00]{of=(md.currency(md.doc.fees.small, "USD"))} |
| Large | {{md.doc.plots.large}} | :value[$40.00]{of=(md.currency(md.doc.fees.large, "USD"))} |

:value[3 plots are]{of=(string(md.doc.plots.small + md.doc.plots.large - md.doc.plots.taken) + " " + md.plural(md.doc.plots.small + md.doc.plots.large - md.doc.plots.taken, "plot is", "plots are"))} still free. Call {{md.doc.editor.name}} at {{md.doc.editor.phone}} to take one.
