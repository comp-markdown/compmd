---
md:
  import:
    - from: ./reviews.ts
      names: [reviews]
  export:
    pack: "@you/reviews"
    version: "1.0.0"
    names: [reviews]
---

What other readers thought of a book, from the app's data: permissive, and given `host`.

```compmd component=reviews signature
params:
  book: string
```
