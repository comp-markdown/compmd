---
md:
  data:
    - from: ./books.csv
      as: books
      schema: {title: string, pages: numbr}
---

::: each {in=(md.data.books) as=b join=list}
- {{b.title}}
::: /each
