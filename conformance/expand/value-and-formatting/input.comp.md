---
price: 12
released: "2026-10-21"
count: 3
md:
  lang: en
---

The annual plan is :value[$120.00/year]{of=(md.currency(md.doc.price * 10, "USD") + "/year")}.

Released :value[October 21, 2026]{of=(md.date(md.doc.released, "long"))}, with :value[3 items]{of=(string(md.doc.count) + " " + md.plural(md.doc.count, "item", "items"))}.

::: value {of=(md.number(0.183 * 100, 0) + "%")}
::: /value
