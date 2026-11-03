---
md:
  import:
    - from: ./Stars.tsx
      names: [{ name: Stars, as: stars }]
    - from: ./Spoiler.svelte
      names: [{ name: Spoiler, as: spoiler }]
  export:
    pack: "@you/app"
    version: "1.0.0"
    names: [stars, spoiler]
---

The app's own components, built with it and trusted: they mount in its page.

```compmd component=stars signature
params:
  value: input<number>
  of: number = 5
```

```compmd component=spoiler signature
params:
  label: string
slots: [default]
```
