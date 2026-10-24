---
md:
  version: 1
  import:
    - from: ./weekly-sales.js
      names: [weekly-sales]
sales:
  - {week: 1, total: 120}
  - {week: 2, total: 135}
---

```compmd component=weekly-sales signature
params:
  rows: array
```

:: weekly-sales[Weekly sales: 120, then 135] {rows=(md.doc.sales) width=50%}
