---
md:
  inputs:
    size: small | large = small
---

```compmd component=form
slots: [default]
---
{{md.slot}}
```

```compmd component=size-picker
---
:: select-input[Size: small] {bind=size}
```

::: form
:: select-input[Size: small] {bind=size}
::: /form

:: size-picker[Size]
