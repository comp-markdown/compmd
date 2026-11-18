---
md:
  import:
    - from: ./Stars.tsx
      names: [{ name: Stars, as: stars }]
  export:
    pack: "@example/app"
    version: "1.0.0"
    names: [stars]
---

The app's own components, built with it and trusted: they mount in its page.

```compmd component=stars signature
params:
  value: input<number>
  of: number = 5
```
