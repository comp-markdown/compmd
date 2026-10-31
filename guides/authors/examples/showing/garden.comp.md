---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
jobs: [Turn the compost, Mend the gate by plot 4, Plant the garlic]
helpers: [Maya, Tomás, Jun]
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic}
  - {number: 2, gardener: Tomás Reyes, crop: kale}
  - {number: 3, gardener: "", crop: ""}
  - {number: 4, gardener: Priya Nair, crop: leeks}
  - {number: 5, gardener: "", crop: ""}
news: []
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

## Work day

Saturday, November 7, from nine to noon. On the list:

::: each {in=(md.doc.jobs) as=job join=list}

1. {{job}}

::: /each

Thanks to last month's helpers:

::: each {in=(md.doc.helpers) as=helper join=inline sep=", " last=" and "}

**{{helper}}**

::: /each

## The plots

::: each {in=(md.doc.plots.filter(p, p.gardener != "")) as=p}

::: box {padding=s border=subtle border-radius=s}

**Plot {{p.number}}**: {{p.gardener}}, growing {{p.crop}}.

::: /box

::: /each

::: if {when=(md.doc.plots.exists(p, p.gardener == ""))}

:value[2 plots are free]{of=(string(size(md.doc.plots.filter(p, p.gardener == ""))) + " " + md.plural(size(md.doc.plots.filter(p, p.gardener == "")), "plot is", "plots are") + " free")} for next year. Ask Maya to take one.

:: #else

Every plot is taken for next year. Ask Maya to put you on the waiting list.

::: /if

::: each {in=(md.doc.news) as=item join=list}

- {{item}}

:: #before

## From the committee

:: #empty

No news from the committee this month.

::: /each
