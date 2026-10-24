---
md:
  version: 1
  tz: Asia/Tokyo
author: {name: Ursula K. Le Guin, born: 1929}
authors:
  - {name: Ana Lima}
  - {name: Kenji Sato}
created: "2026-10-20T23:30:00Z"
meta: {pages: 320}
---

By {{md.doc.author}}, born {{md.doc.author.born}}.

Written by :value[Ana Lima and Kenji Sato]{of=(md.list(md.doc.authors))}.

::: each {in=(md.doc.authors) as=a join=inline sep=", "}
{{a}}
::: /each

Created on :value[2026-10-21]{of=(md.dateOf(timestamp(md.doc.created)))} in Tokyo.

A map without a name is not written: {{md.doc.meta}}.
