---
md:
  version: 1
title: The first frost
date: 2026-11-20
summary: The frost came early this year. What to cover tonight, and what to pick before the weekend.
tags: [news, winter]
pick: [Courgettes, Beans, Tomatoes]
cover: [Leeks, Kale, Parsnips]
---

The forecast says minus three tonight, two weeks before anyone expected it.

::: row {gap=m}
::: box {padding=m border=subtle border-radius=m}
**Pick today**

::: each {in=(md.doc.pick) as=crop join=list}
- {{crop}}
::: /each
::: /box
::: box {padding=m border=subtle border-radius=m}
**Cover tonight**

::: each {in=(md.doc.cover) as=crop join=list}
- {{crop}}
::: /each
::: /box
::: /row

The fleece is in the tool shed, on the shelf by the door. Take what you need, and bring it back on Sunday.
