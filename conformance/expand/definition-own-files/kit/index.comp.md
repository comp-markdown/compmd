---
md:
  export:
    pack: "@acme/kit"
    version: "1.4.2"
    names: "*"
---

```compmd component=card
---
![A star](./icons/star.svg) [The guide](./guide.comp.md#start)
```

```compmd component=link-card
params:
  href: string
---
[Read on]({{href}})
```

```compmd component=teaser
params:
  href: string
---
:: link-card[Read on] {href=(href)}
```

```compmd component=gallery
params:
  rows: array
---
::: each {in=(rows) as=r}
![{{r.alt}}]({{r.image}})
::: /each
```

```compmd component=stray
---
![Outside](../outside.svg)
```
