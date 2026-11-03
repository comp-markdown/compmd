---
title: Releases this quarter
md:
  import:
    - from: ./parts/notice.comp.md
      names: [notice]
  data:
    - from: ./releases.csv
      as: releases
      schema: {version: string, date: string, changes: number}
---

# {{md.doc.title}}

::: notice {title="Three releases" border=subtle}
:value[24 changes]{of=(md.number(md.sum(md.data.releases.map(r, r.changes)), 0) + " changes")} in all, each listed below.
::: /notice

::: each {in=(md.data.releases) as=r join=list}
- **{{r.version}}**, {{r.date}}: {{r.changes}} changes
::: /each
