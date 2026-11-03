---
md:
  data:
    - from: https://api.example.com/sales.json
      as: sales
    - from: https://api.example.com/team.json
      as: team
      fallback: ./team.json
    - from: host:tasks
      as: tasks
---

::: each {in=(md.data.sales) as=s join=list}
- {{s.region}}
:: #empty
Loading the sales.
::: /each

Team: {{md.data.team.lead}}.

::: each {in=(md.data.tasks) as=t join=list}
- {{t.title}}
:: #empty
No tasks yet.
::: /each
