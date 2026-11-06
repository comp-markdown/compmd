---
md:
  import:
    - from: ./shelf.js
      names: [shelf]
  data: [{from: ./club.csv, as: books}]
  inputs:
    book: string = "Klara and the Sun"
---

# Thursday Book Club

We meet on the last Thursday of the month, at Ana's.
Pick a cover to see what we made of it.

:: shelf[] {books=(md.data.books) picked=book}

::: each {in=(md.data.books.filter(b, b.title == md.input.book)) as=b}
**{{b.title}}**, {{b.author}}. {{b.verdict}}

::: if {when=(b.status == "read")}
The club gave it :rating[4 of 5]{value=(b.rating)}

:: #else
Not rated yet.
::: /if
::: /each

```compmd component=shelf signature
params:
  books: array
  picked: input<string>
```
