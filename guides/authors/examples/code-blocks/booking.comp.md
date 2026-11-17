---
md:
  version: 1
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
title: Elm Street Garden
---

# Booking a plot online

From March you can book a plot from your phone. This is the first sketch of the screen: tell us at the next work day what is missing.

```wireframe {alt="The booking screen on a phone: a half or a full plot, the month you start, a key to the tool shed, and a Book button"}
window "Book a plot" device=phone:
  header:
    text "Elm Street Garden" bold
  section "Your plot":
    radio "Half plot" group="size" selected
    radio "Full plot" group="size"
    combo "Starting" value="March" options="March, April, May"
  checkbox "A key to the tool shed" label-right
  button "Book" primary id=book
annotation "You get an email to confirm" target="book" position=bottom
```

The paper list by the gate stays until everyone who wants a plot has one.
