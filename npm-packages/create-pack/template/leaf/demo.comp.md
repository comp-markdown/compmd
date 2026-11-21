---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

# __component__

Each example shows its source, then what it draws: one for each param and use, so a reader sees what the component offers.

**A use:**

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[Hello, from __component__] {value="Hello"}
````

**Sized by the document,** wider:

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[Hello, wider] {value="Wider" width=20rem}
````

**In a short box,** which it is fitted in:

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[Hello, in a short box] {value="Short" width=100% height=3rem}
````

**Covering its box,** `object-fit=cover`:

````compmd {render=both}
---
md:
  import:
    - from: "__pack__@0"
      names: [__component__]
---

:: __component__[Hello, covering its box] {value="Cover" width=16rem height=8rem object-fit=cover}
````
