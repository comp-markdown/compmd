```compmd component=outer
---
::: inner
Inside.
::: /inner
```

```compmd component=inner
slots: [default]
---
:: outer[Again]

{{md.slot}}
```

:: outer[Start]
