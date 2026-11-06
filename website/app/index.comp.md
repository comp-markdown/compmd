---
md:
  import:
    - from: ./Stars.svelte
      names: [{ name: Stars, as: stars }, { name: Stars, as: rating }]
  export:
    pack: "@site/app"
    version: "1.0.0"
    names: [stars, rating]
---

```compmd component=stars signature
params:
  value: input<number>
  of: number = 5
```

```compmd component=rating signature
params:
  value: number
  of: number = 5
  readonly: boolean = true
```
