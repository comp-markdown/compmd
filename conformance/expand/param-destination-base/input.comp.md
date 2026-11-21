```compmd component=link-card
params:
  title: string
  href: string
  icon: string = "star"
  rows: array = []
---
[{{title}}]({{href}}) ![](./icons/{{icon}}.svg)

::: each {in=(rows) as=walk}
![{{walk.alt}}]({{walk.image}})
::: /each
```

:: link-card[The harbour at dusk] {title="The harbour at dusk" href="harbour.comp.md" rows=([{"alt": "The quay", "image": "images/quay.jpg"}])}
