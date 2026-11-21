```compmd component=columns {display=grid gap=m grid-template-columns=(md.slots.middle ? "1fr 1fr 1fr" : "1fr 1fr")}
slots: [default, middle, end]
---
::: box
{{md.slot}}
::: /box

::: if {when=(md.slots.middle)}
::: box
{{md.slot.middle}}
::: /box
::: /if

::: box
{{md.slot.end}}
::: /box
```

::: columns
The first column.

:: #end
The last.
::: /columns

::: columns
The first.

:: #middle
The middle.

:: #end
The last.
::: /columns
