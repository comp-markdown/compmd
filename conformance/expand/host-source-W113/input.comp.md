---
md:
  data:
    - from: host:tasks
      as: tasks
      schema: {title: string}
---

::: each {in=(md.data.tasks) as=t join=inline sep=", "}
{{t.title}}
::: /each
