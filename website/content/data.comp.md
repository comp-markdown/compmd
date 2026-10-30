---
md:
  data:
    - from: ./books.csv
      as: books
      schema: {title: string, author: string, year: number, pages: number}
---

::: each {in=(md.sortBy(md.data.books, "year")) as=b join=list}
- *{{b.title}}*, {{b.author}} ({{b.year}})
::: /each

:value[863 pages]{of=(md.number(md.sum(md.data.books.map(b, b.pages)), 0) + " pages")} in all.
