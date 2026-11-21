```compmd component=figure
params:
  src: string
---
![{{md.fallback}}]({{src}})

{{md.fallback}}
```

```compmd component=badge {padding-inline=s border-radius=full background=accent}
---
{{md.fallback}}
```

```compmd component=panel
slots: [default]
---
{{md.fallback}} {{md.slot}}
```

:: figure[Boats at the quay, the lamps *just* lit.] {src="harbour.jpg"}

The new :badge[beta] search.

::: panel
A container has no fallback of its own.
::: /panel
