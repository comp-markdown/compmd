---
md:
  inputs:
    region: all | eu | us = all
    since: date = (md.addDays(md.today(), -7))
    limit: number = 5
  data:
    - from: ./sales-{{md.input.region}}.json
      as: sales
---

:: select-input[Region: all] {bind=region}
:: date-input[Since] {bind=since}

Region {{md.input.region}}, since {{md.input.since}}, limit {{md.input.limit}}: {{md.data.sales.total}}
