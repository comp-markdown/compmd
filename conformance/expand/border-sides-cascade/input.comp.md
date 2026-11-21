```compmd component=timeline {border-inline-start=strong padding-inline-start=m}
slots: [default]
---
{{md.slot}}
```

```compmd component=framed {border=subtle}
slots: [default]
---
{{md.slot}}
```

::: timeline
The fence's start border.
::: /timeline

::: timeline {border=none}
The caller's `border` covers the fence's side.
::: /timeline

::: framed {border-inline-start=strong}
The fence's `border`, and the caller's start side over it.
::: /framed
