---
md:
  version: 1
  export:
    names: [plot-card, notice]
garden: Elm Street Garden
---

# The garden's components

The components every issue of the {{md.doc.garden}} newsletter uses.

```compmd component=plot-card {padding=s border=subtle border-radius=m}
params:
  number: number
  gardener: string?
  crop: string = "something new"
  size: small | large = small
---
# Plot {{number}}, {{size}}

::: if {when=(gardener == null)}

Free for next year: ask at the {{md.doc.garden}} shed.

:: #else

**{{gardener}}**, growing {{crop}}.

::: /if
```

```compmd component=notice {padding=m border-radius=m background=(tone == "urgent" ? "accent" : "subtle")}
params:
  title: string
  tone:
    type: plain | urgent
    default: plain
    doc: An urgent notice has the accent background.
slots: [default]
---
# {{title}}

{{md.slot}}
```
