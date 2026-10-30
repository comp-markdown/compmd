---
md:
  data:
    - from: ./books.csv
      as: books
      schema: {title: string, author: string, year: number, pages: number}
  inputs:
    since: number = 1970
---

:: number-input[Published since: 1970] {bind=since}

::: each {in=(md.data.books.filter(b, b.year >= md.input.since)) as=b join=list}
- *{{b.title}}* ({{b.year}})

:: #empty
No book is that recent.
::: /each
