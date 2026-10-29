---
title: Sales, January to September 2026
months: {"01": Jan, "02": Feb, "03": Mar, "04": Apr, "05": May, "06": Jun, "07": Jul, "08": Aug, "09": Sep}
regions: {all: all regions, eu: Europe, us: the United States, apac: Asia-Pacific}
md:
  version: 1
  inputs:
    region: all | eu | us | apac = all
  data:
    - from: ./sales.csv
      as: sales
      schema: {month: string, region: string, product: string, units: number, revenue: number}
    - from: ./milestones.yaml
      as: milestones
      schema: {date: date, title: string, kind: string}
---

# {{md.doc.title}}

Three plans sold in three regions. Pick a region, or click its column in the chart: every figure below follows it.

:: select-input[Region: all] {bind=region}

```compmd component=kpi {display=flex flex-direction=column gap=s padding=m border=subtle border-radius=m}
params:
  label: string
  value: string
  note: string
---
{{label}}, {{note}}

**{{value}}**
```

```compmd component=sales {display=contents}
params:
  rows: array
  place: string
  months: object
headings: absolute
---
::: grid {min=11rem gap=m}
:: kpi[Revenue] {label="Revenue" value=(md.currency(md.sum(rows.map(r, r.revenue)), "USD")) note=("in " + place)}
:: kpi[Units] {label="Units sold" value=(md.number(md.sum(rows.map(r, r.units)), 0)) note=(md.number(double(md.sum(rows.map(r, r.units))) / 9.0, 0) + " a month")}
:: kpi[Average price] {label="Average price" value=(md.currency(md.sum(rows.map(r, r.revenue)) / md.sum(rows.map(r, r.units)), "USD")) note="per unit"}
::: /grid

::: grid {md:grid-template-columns="3fr 2fr" gap=l}
::: stack {gap=s}
## Revenue by month

:: area-chart[Monthly revenue in {{place}}, January to September] {unit="USD" rows=(md.groupBy(rows, "month").map(m, {"label": months[m.substring(5, 7)], "value": md.sum(md.groupBy(rows, "month")[m].map(r, r.revenue))}))}
::: /stack

::: stack {gap=s}
## By plan

:: pie-chart[Revenue by plan in {{place}}] {unit="USD" rows=(md.groupBy(rows, "product").map(p, {"label": p, "value": md.sum(md.groupBy(rows, "product")[p].map(r, r.revenue))}))}
::: /stack
::: /grid

## Every month

:: data-table[Units and revenue by month in {{place}}] {unit="USD" cols=([{"key": "month", "label": "Month"}, {"key": "units", "label": "Units"}, {"key": "revenue", "label": "Revenue"}]) rows=(md.groupBy(rows, "month").map(m, {"month": months[m.substring(5, 7)], "units": md.sum(md.groupBy(rows, "month")[m].map(r, r.units)), "revenue": md.sum(md.groupBy(rows, "month")[m].map(r, r.revenue))}))}
```

::: grid {md:grid-template-columns="3fr 2fr" gap=l}
::: stack {gap=s}
## Revenue by region

:: bar-chart[Revenue by region, January to September: the United States leads] {unit="USD" selected=region rows=(md.groupBy(md.data.sales, "region").map(g, {"label": g, "value": md.sum(md.groupBy(md.data.sales, "region")[g].map(r, r.revenue))}))}
::: /stack

::: stack {gap=s}
## Milestones

:: timeline[Releases, campaigns and partners through the year] {events=(md.data.milestones) kinds=(["release", "campaign", "partner"])}
::: /stack
::: /grid

:: sales[Sales in the region picked above] {rows=(md.data.sales.filter(s, md.input.region == "all" || s.region == md.input.region)) place=(md.doc.regions[md.input.region]) months=(md.doc.months)}
