```compmd component=panel {padding=(pad) background=(surface)}
params:
  pad: space = m
  surface: surface = subtle
slots: [default]
---
{{md.slot}}
```

::: panel {pad=l}
A `space` and a `surface` param, passed on to the box: both are CEL strings.
::: /panel
