---
md:
  version: 1
  data:
    - from: ./plots.csv
      as: plots
      schema:
        number: {from: Plot, type: number}
        gardener: {from: Gardener, type: string?}
        crop: {from: Crop, type: string?}
        size: {from: "Size (m2)", type: number}
    - from: ./workdays.yaml
      as: workdays
      schema: {date: date, jobs: array}
    - from: ./club/about.comp.md
      as: club
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Work days

::: each {in=(md.data.workdays) as=day}

**:value[Saturday, November 7, 2026]{of=(md.date(day.date, "full"))}**, from nine to noon: :value[turn the compost, mend the gate by plot 4, and plant the garlic]{of=(md.list(day.jobs))}.

::: /each

## The plots

::: each {in=(md.data.plots.filter(p, p.gardener != null)) as=p join=list}

- **Plot {{p.number}}**, {{p.size}} m²: {{p.gardener}}, growing {{p.crop}}

::: /each

:value[2 plots are free]{of=(string(size(md.data.plots.filter(p, p.gardener == null))) + " " + md.plural(size(md.data.plots.filter(p, p.gardener == null)), "plot is", "plots are") + " free")} for next year. A small plot is :value[$25.00]{of=(md.currency(md.data.club.fees.small, "USD"))} a year, a large one :value[$40.00]{of=(md.currency(md.data.club.fees.large, "USD"))}. Call {{md.data.club.contact}} at {{md.data.club.contact.phone}} to take one.
