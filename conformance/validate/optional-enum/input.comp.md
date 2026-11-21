```compmd component=qr-code
params:
  value: string
  level: L | M | H | Q?
---
{{value}}
```

An optional enum may be left out, and takes only its members when given.

:: qr-code[Our site] {value=site}

:: qr-code[Our site, sturdier] {value=site level=H}

:: qr-code[Our site, at a level it hasn't] {value=site level=X}
