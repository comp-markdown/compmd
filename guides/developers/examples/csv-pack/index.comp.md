---
md:
  import:
    - from: ./dist/csv.js
      names: [csv]
  export:
    pack: "@you/csv"
    version: "0.1.0"
    names: [csv]
---

# @you/csv

`csv` draws every ` ```csv ` block of a document that imports it as a table; a reader without it sees the values. Written as a leaf, its `code` is a param, for rows made from data: `:: csv[The totals] {code=("total,12\nopen,3")}`.

```compmd component=csv signature
renders: csv
params:
  header: boolean = true
```
