---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
news:
  - The water goes off on November 15. Empty your cans and hoses before then.
  - The seed swap is on December 5, in the library's back room.
  - Tomás has spare kale seedlings, free to a good bed.
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

::: grid {gap=m md:grid-template-columns="2fr 1fr" md:gap=l}

::: box

## News

::: each {in=(md.doc.news) as=item join=list}

- {{item}}

::: /each

::: /box

::: box {padding=m md:padding=l border-radius=m background=subtle}

## Work day

Saturday, November 7, from nine to noon. Bring gloves; we bring the tea.

::: /box

::: /grid

## The plots

::: box {hide=md}

- **Plot 1**: Maya Okafor, garlic.
- **Plot 2**: Tomás Reyes, kale.
- **Plot 4**: Priya Nair, leeks.

Plots 3 and 5 are free.

::: /box

::: box {show=md}

| Plot | Gardener | Growing | Since | Next year |
| ---: | --- | --- | --- | --- |
| 1 | Maya Okafor | garlic | 2019 | beans |
| 2 | Tomás Reyes | kale | 2023 | roots |
| 3 | free | | | |
| 4 | Priya Nair | leeks | 2021 | potatoes |
| 5 | free | | | |

::: /box
