---
md:
  import:
    - from: ./dist/rating.js
      names: [rating]
    - from: ./dist/lanes.js
      names: [lanes]
  export:
    pack: "@you/rating"
    version: "0.1.0"
    names: [rating, lanes]
---

# @you/rating

`rating`, a leaf the reader sets, bound to a number input: `:: rating[3 of 5 stars] {value=stars}`.

```compmd component=rating signature
params:
  value: input<number>
  max: number = 5
  size: small | large = small
  color: color?
```

`lanes`, a container whose default slot is a template, once per record, in the lane of its `by` field.

```compmd component=lanes signature
params:
  rows: array
  by: string
  as: name
slots:
  default: {each: rows, as: as}
```
