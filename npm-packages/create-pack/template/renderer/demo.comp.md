---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

# __component__

```__language__ {alt="Two lines of __language__"}
first line
second line
```

The same component as a leaf, its code a param:

:: __component__[One line of __language__] {code="one line"}

Sized by the document: narrower, and a height its lines scroll in.

:: __component__[One narrow line of __language__] {code="one narrow line" width=16rem}

```__language__ {alt="Six lines of __language__, in a box three lines tall" height=5rem scroll}
one
two
three
four
five
six
```
