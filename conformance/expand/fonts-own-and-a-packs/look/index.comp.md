---
md:
  export:
    pack: "@acme/look"
    version: "1.0.0"
    names: "*"
    fonts: "*"
  fonts:
    harbour-serif:
      files:
        - { from: ./fonts/harbour-serif.woff2 }
        - { from: ./fonts/harbour-serif-italic.woff2, style: italic }
---

```compmd component=masthead {font-family=harbour-serif font-style=italic}
---
{{md.fallback}}
```
