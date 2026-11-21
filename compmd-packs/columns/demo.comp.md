---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

Each example of `columns` shows its source, then what it draws.

# Two columns

The first column is what you write first; `:: #end` starts the second.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns
**The harbour at dusk.** A walk along the old quay, an hour before the boats come in.

:: #end
**The old orchard.** Apples in October, and a bench at the top of the hill.
::: /columns
````

# Three columns

`:: #middle` starts a middle one.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns
**Morning**: the harbour.

:: #middle
**Noon**: the orchard.

:: #end
**Evening**: the canal.
::: /columns
````

# A wide column

`split=wide-start` or `wide-end` makes one column twice the other: a recipe's method beside its ingredients.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-end}
## Ingredients

- 4 apples
- 100 g butter
- 100 g sugar

:: #end
## Method

Peel and slice the apples, and lay them in a buttered dish.

Rub the butter into the flour and sugar, scatter it over, and bake for 40 minutes.
::: /columns
````

# A wide middle

`split=wide-middle`, with three columns: a story between two notes.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-middle}
*From the harbour master*

:: #middle
The quay floods at high water, and has for three hundred years. This winter the new wall will keep the lamps dry.

:: #end
*Tide times inside*
::: /columns
````

# Right to left

Start and end follow the text's direction: in Arabic the first column is on the right. Here the columns are in Arabic in an English page, so they say so with `lang` and `dir`.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/columns@0"
      names: [columns]
---

::: columns {split=wide-start lang=ar dir=rtl}
**الميناء عند الغروب.** نزهة على طول الرصيف القديم.

:: #end
**البستان القديم.** التفاح في أكتوبر.
::: /columns
````
