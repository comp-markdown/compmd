---
md:
  data:
    - from: ./team.yaml
      as: team
    - from: ./site.comp.md
      as: site
    - from: https://status.example.com/today.json
      as: status
    - from: https://status.example.com/down.json
      as: down
      fallback: ./cached.json
    - from: ./missing.json
      as: missing
    - from: host:tasks
      as: tasks
---

{{md.data.site.title}}: {{md.data.status.summary}} / {{md.data.down.summary}}

::: each {in=(md.data.team) as=p join=inline sep=", "}
{{p.name}}
::: /each

::: each {in=(md.data.missing) as=m}
{{m}}
:: #empty
No data.
::: /each

::: each {in=(md.data.tasks) as=t join=inline sep=", "}
{{t.title}}
::: /each
