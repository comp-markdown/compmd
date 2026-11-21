---
md:
  import:
    - from: "@acme/kit"
      names: [card, teaser, gallery, stray]
    - from: ./parts/notes.comp.md
      names: [note]
---

:: card[A star and the guide]

:: teaser[Read on] {href=harbour.comp.md}

:: gallery[The quay] {rows=([{"alt": "The quay", "image": "images/quay.jpg"}])}

:: note[A note and home]

:: stray[Outside]

![Our own](./own.svg)
