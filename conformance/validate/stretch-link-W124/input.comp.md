```compmd component=link-card {stretch-link padding=m}
params:
  title: string
  href: string
---
### [{{title}}]({{href}})
```

:: link-card[The harbour at dusk] {title="The harbour at dusk" href="harbour.comp.md"}

::: box {stretch-link}
A box with [a link](walks.comp.md) in its text.
::: /box

::: box {stretch-link}
A box with no link.
::: /box

::: box {md:stretch-link}
A breakpoint.
::: /box
