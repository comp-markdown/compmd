```compmd component=section
params:
  title: string
slots: [default]
---
# {{title}}

## Details

{{md.slot}}
```

```compmd component=cover
params:
  title: string
headings: absolute
---
# {{title}}
```

# Release

::: section {title="Renderer"}
Faster.
::: /section

## Changes

::: section {title="Config"}
New format.
::: /section

:: cover[Title page] {title="RayPlay"}
