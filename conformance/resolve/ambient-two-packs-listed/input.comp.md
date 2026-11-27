---
due: "1 December"
md:
  import:
    - from: "@acme/fields"
      names: [field]
---

Listing `field` from one of the two packs the host provides chooses it: Acme's, with nothing reported.

:: field[Due: {{md.doc.due}}] {value=(md.doc.due)}
