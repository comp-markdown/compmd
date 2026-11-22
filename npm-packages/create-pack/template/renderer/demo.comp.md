---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

Each example of `__component__` shows its source, then what it draws: one for each param and use, so a reader sees what the component offers.

# A code block it renders

Its `alt` is the text alternative.

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

```__language__ {alt="Two lines of __language__"}
first line
second line
```
````

# The same component as a leaf

Its code is a param.

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[One line of __language__] {code="one line"}
````

# Sized by the document

Narrower.

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[One narrow line of __language__] {code="one narrow line" width=16rem}
````

# A height its lines scroll in

`height=5rem scroll`.

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

```__language__ {alt="Six lines of __language__, in a box three lines tall" height=5rem scroll}
one
two
three
four
five
six
```
````
