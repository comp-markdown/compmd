---
md:
  version: 1
  data:
    - from: ./harvest.csv
      as: harvest
      schema: {date: date, plot: number, crop: string, kg: number}
title: Elm Street Garden
issue: November 2026
workdays: ["2026-11-07", "2026-11-21", "2026-12-05"]
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}, for the week of :value[October 25, 2026]{of=(md.date(md.startOf(md.today(), "week"), "long"))}.

## The next work day

::: each {in=(md.slice(md.doc.workdays.filter(d, d >= md.today()), 0, 1)) as=next}

:value[Saturday, November 7, 2026]{of=(md.date(next, "full"))}, in :value[7 days]{of=(string(md.daysBetween(md.today(), next)) + " " + md.plural(md.daysBetween(md.today(), next), "day", "days"))}. After it: :value[November 21, 2026 and December 5, 2026]{of=(md.list(md.doc.workdays.filter(d, d > next).map(d, md.date(d, "long"))))}.

:: #empty

No more work days this year. See you in the spring.

::: /each

## The harvest

Since :value[September 12]{of=(md.date(md.sort(md.data.harvest.map(h, h.date))[0], "long"))}, the plots gave :value[30.0 kg]{of=(md.number(md.sum(md.data.harvest.map(h, h.kg)), 1) + " kg")} in :value[7 pickings]{of=(string(size(md.data.harvest)) + " " + md.plural(size(md.data.harvest), "picking", "pickings"))}, :value[4.3 kg]{of=(md.number(md.sum(md.data.harvest.map(h, h.kg)) / size(md.data.harvest), 1) + " kg")} each on average. The biggest was :value[9.0 kg]{of=(md.number(md.max(md.data.harvest.map(h, h.kg)), 1) + " kg")} of :value[squash]{of=(md.reverse(md.sortBy(md.data.harvest, "kg"))[0].crop)}.

::: each {in=(md.sort(md.unique(md.data.harvest.map(h, h.crop)))) as=crop join=list}

- **{{crop}}**: :value[7.7 kg]{of=(md.number(md.sum(md.groupBy(md.data.harvest, "crop")[crop].map(h, h.kg)), 1) + " kg")} from :value[2 pickings]{of=(string(size(md.groupBy(md.data.harvest, "crop")[crop])) + " " + md.plural(size(md.groupBy(md.data.harvest, "crop")[crop]), "picking", "pickings"))}

::: /each

The last three:

::: each {in=(md.slice(md.reverse(md.sortBy(md.data.harvest, "date")), 0, 3)) as=h join=list}

1. :value[October 24]{of=(md.date(h.date, "long"))}: {{h.crop}} from plot {{h.plot}}, {{h.kg}} kg

::: /each

Bring your spare :value[beans, kale, leeks, or squash]{of=(md.list(md.sort(md.unique(md.data.harvest.map(h, h.crop))), "or"))} to the swap table by the gate.
