---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
jobs: [Turn the compost, Mend the gate by plot 4, Plant the garlic]
plots:
  - {number: 1, gardener: Maya Okafor, crop: garlic}
  - {number: 2, gardener: Tomás Reyes, crop: kale}
  - {number: 3, gardener: "", crop: ""}
  - {number: 4, gardener: Priya Nair, crop: leeks}
  - {number: 5, gardener: "", crop: ""}
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: stack {gap=s}

::: box {padding=m border-radius=m background=accent}

**The water goes off on November 15.** Empty your cans and hoses before then.

::: /box

::: box {padding=m border-radius=m background=subtle}

**The tools go back in the shed** after the work day. Maya has the key.

::: /box

::: /stack

## Work day

Saturday, November 7, from nine to noon. On the list:

::: row {gap=s}

::: each {in=(md.doc.jobs) as=job}

::: box {padding="s m" border=subtle border-radius=full}

{{job}}

::: /box

::: /each

::: /row

## The plots

::: grid {min=12rem gap=m}

::: each {in=(md.doc.plots) as=p}

::: box {padding=m border=subtle border-radius=m background=(p.gardener == "" ? "accent" : "none")}

### Plot {{p.number}}

::: if {when=(p.gardener != "")}

{{p.gardener}}, growing {{p.crop}}.

:: #else

Free for next year. Ask Maya.

::: /if

::: /box

::: /each

::: /grid
