---
md:
  import:
    - from: ./Stars.svelte
      names: [{ name: Stars, as: stars }]
  export:
    pack: "@site/app"
    version: "1.0.0"
    names: [stars]
---

```compmd component=stars signature
params:
  value: input<number>
  of: number = 5
```
