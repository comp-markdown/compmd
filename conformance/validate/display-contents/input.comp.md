```compmd component=tiles {display=contents padding=s}
slots: [default]
---
{{md.slot}}
```

```compmd component=late {md:display=contents}
slots: [default]
---
{{md.slot}}
```

```jsx component=chart {display=flex height=16rem}
export default function Chart() { return null; }
```

::: tiles {.wide width=50% #main}
Text.
::: /tiles

::: box {display=contents}
Text.
::: /box
