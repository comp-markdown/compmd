---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic, x: "6%", y: "10%"}
  - {number: 2, gardener: Tomás Reyes, crop: kale, x: "38%", y: "10%"}
  - {number: 3, gardener: "", crop: "", x: "70%", y: "10%"}
  - {number: 4, gardener: Priya Nair, crop: leeks, x: "6%", y: "55%"}
  - {number: 5, gardener: "", crop: "", x: "38%", y: "55%"}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: grid {gap=m grid-template-areas="'map' 'plots' 'notes'" md:grid-template-columns="2fr 1fr" md:grid-template-areas="'map notes' 'plots notes'"}

::: box {grid-area=map aspect-ratio=3/2 border=subtle border-radius=m background=subtle}

::: each {in=(md.doc.plots) as=p}

::: box {inset-inline-start=(p.x) inset-block-start=(p.y) width=26% padding=s border-radius=s background=(p.gardener == "" ? "accent" : "strong")}

**{{p.number}}**

::: /box

::: /each

::: box {inset-inline-end=4% inset-block-end=8% padding=s border=strong border-radius=s}

Shed

::: /box

::: box {inset-inline-start=0 inset-block-end=0 padding=s}

Gate

::: /box

::: /box

::: stack {grid-area=plots gap=s}

## The plots

::: each {in=(md.doc.plots) as=p}

::: box {padding=s border=subtle border-radius=s}

::: if {when=(p.gardener != "")}

**Plot {{p.number}}**: {{p.gardener}}, growing {{p.crop}}.

:: #else

**Plot {{p.number}}** is free for next year.

::: /if

::: /box

::: /each

::: /stack

::: stack {grid-area=notes gap=s align-self=start md:sticky=top}

## Notes

::: box {padding=m border-radius=m background=accent}

**The water goes off on November 15.**

::: /box

::: box {padding=m border-radius=m background=subtle}

**Work day:** Saturday, November 7, from nine to noon. Mend the gate first.

::: /box

::: /stack

::: /grid
