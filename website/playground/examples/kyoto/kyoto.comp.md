---
title: Kyoto in four days
days:
  - {n: 1, name: Higashiyama, walk: "Kiyomizu-dera, Sannenzaka, Yasaka Shrine", eat: "Yudofu near Nanzen-ji", tag: temples}
  - {n: 2, name: Arashiyama, walk: "Bamboo grove, Okochi Sanso, Tenryu-ji", eat: "Soba by the Katsura river", tag: gardens}
  - {n: 3, name: Fushimi and Uji, walk: "Fushimi Inari's gates, then Byodo-in", eat: "Matcha in Uji", tag: shrines}
  - {n: 4, name: Northern hills, walk: "Kurama to Kibune over the ridge", eat: "Kawadoko over the stream", tag: hiking}
md:
  version: 1
---

```compmd component=walk-eat {display=contents}
params:
  walk: string
  eat: string
---
**Walk:** {{walk}}.

**Eat:** {{eat}}.
```

```compmd component=badge {padding="0 s" border=subtle border-radius=full background=subtle}
params:
  text: string
---
{{text}}
```

::: box {aspect-ratio=21/9 padding=l border-radius=l background=accent}
# {{md.doc.title}}

Temples at dawn, gardens after the rain, a ridge walk between two mountain villages.

::: box {inset-inline-end=1rem inset-block-end=1rem width=14rem padding=m border-radius=m background=strong}
**Best in** late October and November, when the maples turn.
::: /box

:: badge[4 days] {text="4 days" inset-inline-end=1rem inset-block-start=1rem}
::: /box

::: grid {gap=l grid-template-areas="'plan' 'notes' 'pack'" md:grid-template-columns="2fr 1fr" md:grid-template-areas="'plan notes' 'pack notes'" lg:grid-template-columns="1fr 2fr 1fr" lg:grid-template-areas="'pack plan notes'"}

::: stack {grid-area=plan gap=m}
## The plan

::: each {in=(md.doc.days) as=d}
::: box {padding=m border=subtle border-radius=m}
::: row {gap=s align-items=center}
### Day {{d.n}}: {{d.name}}

:: badge[{{d.tag}}] {text=(d.tag)}
::: /row

::: grid {md:grid-template-columns="1fr 1fr" gap=s}
:: walk-eat[Walk: {{d.walk}}. Eat: {{d.eat}}.] {walk=(d.walk) eat=(d.eat)}
::: /grid
::: /box
::: /each
::: /stack

::: stack {grid-area=notes gap=s sticky=top}
## Notes

- Buy an ICOCA card at the station; buses and trains take it.
- Most temples open at 8:30 and close by 17:00.
- Leave the bags at the hotel: the city is best on foot.

::: box {show=md padding=m border-radius=m background=subtle}
On a wide screen this column stays in view as you scroll the plan.
::: /box
::: /stack

::: stack {grid-area=pack gap=s}
## Pack

::: row {gap=s}
:: badge[walking shoes] {text="walking shoes"}
:: badge[an umbrella] {text="an umbrella"}
:: badge[a coin purse] {text="a coin purse"}
:: badge[a light jacket] {text="a light jacket"}
::: /row

::: box {hide=md}
On a phone, the packing list comes last.
::: /box
::: /stack

::: /grid
