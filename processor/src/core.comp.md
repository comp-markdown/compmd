---
md:
  version: 1
  export:
    pack: "@compmd/core"
    version: "1.0.0"
    names: "*"
---

# @compmd/core

The built-ins (spec §9), built into every processor and imported into every document. A signature fence alone is
a component the processor implements.

```compmd component=box signature
params:
  display: block | flex | grid | contents = block
  flex-direction: row | column = row
  gap: space = none
  align-items: start | center | end | stretch = stretch
  justify-content: start | center | end | between | around | space-between | space-around = start
  flex-wrap: wrap | nowrap = nowrap
  grid-template-columns: tracks?
  grid-template-rows: tracks?
  grid-template-areas: string?
slots: [default]
```

```compmd component=stack {display=flex flex-direction=column gap=(gap) align-items=(align_items)}
params:
  gap: space = none
  align-items: start | center | end | stretch = stretch
slots: [default]
---
{{md.slot}}
```

```compmd component=row {display=flex flex-direction=row gap=(gap) align-items=(align_items) justify-content=(justify_content) flex-wrap=(flex_wrap)}
params:
  gap: space = none
  align-items: start | center | end | stretch = stretch
  justify-content: start | center | end | between | around | space-between | space-around = start
  flex-wrap: wrap | nowrap = wrap
slots: [default]
---
{{md.slot}}
```

```compmd component=grid {display=grid gap=(gap) align-items=(align_items) grid-template-columns=(grid_template_columns != null ? grid_template_columns : min != null ? "repeat(auto-fill, minmax(" + min + ", 1fr))" : null) grid-template-rows=(grid_template_rows) grid-template-areas=(grid_template_areas)}
params:
  min: length?
  gap: space = none
  align-items: start | center | end | stretch = stretch
  grid-template-columns: tracks?
  grid-template-rows: tracks?
  grid-template-areas: string?
slots: [default]
---
{{md.slot}}
```

```compmd component=if signature
params:
  when: boolean
slots: [default, else]
```

```compmd component=each signature
params:
  in: array
  as: name
  join: none | list | inline = none
  sep: string = " "
  last: string?
slots:
  default: {each: in, as: as}
  before: {}
  empty: {}
```

```compmd component=value signature
params:
  of: string
```

```compmd component=date-input signature
params:
  bind: input<date>
```

```compmd component=number-input signature
params:
  bind: input<number>
```

```compmd component=text-input signature
params:
  bind: input<string>
```

```compmd component=select-input signature
params:
  bind: input<enum>
```

```compmd component=toggle-input signature
params:
  bind: input<boolean>
```
