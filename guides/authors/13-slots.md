# 13. Slots

A component of your own can wrap content you write each time, as `box` does: a notice around its message, a work day around its plans. That content goes in the component's *slots*. This chapter gives the newsletter a notice with one slot and a work day with three, and shows how a component decides where each part of its body goes.

## The newsletter, with slots

````md snippet=slots/garden.comp.md
---
md:
  version: 1
title: Elm Street Garden
issue: November 2026
---

# {{md.doc.title}}

News from the plots, {{md.doc.issue}}.

```compmd component=notice {padding=m border-radius=m background=subtle}
params:
  title: string
slots: [default]
---
# {{title}}

{{md.slot}}
```

```compmd component=work-day {padding=m border=subtle border-radius=m}
params:
  date: date
  hours: string
slots: [default, jobs, bring]
---
# Work day: :value[Saturday, November 7, 2026]{of=(md.date(date, "full"))}

{{hours}}.

{{md.slot}}

## On the list

{{md.slot.jobs}}

::: box {padding=s border-radius=s background=subtle}

{{md.slot.bring}}

::: /box
```

::: notice {title="The water goes off on November 15"}

Empty your cans and hoses before then. The taps come back on in March.

::: /notice

::: work-day {date=2026-11-07 hours="From nine to noon"}

The last work day of the year, and the garlic goes in.

:: #bring

**Bring** gloves and a trowel; we bring the tea.

:: #jobs

- Turn the compost
- Mend the gate by plot 4
- Plant the garlic

::: /work-day
````

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/authors/slots) in the playground.

The page shows the notice in a shaded box, then the work day: its date as a heading, *From nine to noon*, the line about the garlic, the heading *On the list* with the three jobs, and what to bring in a shaded box at the end, though the text gives it before the jobs.

## Declaring slots

A component takes a body only when its signature lists slots, under `slots`:

```text
slots: [default, jobs, bring]
```

`default` is the slot for the body itself, what you write after the opener. The other names are the parts you start with a slot line, `:: #jobs`. List `default` whenever the component takes a body of its own, as both of these do.

A component without `slots` takes no body, so it is used as a leaf or inline (chapter 12); a body given to it is `F018`.

## Placing slots in the body

In the body, `{{md.slot}}` places the default slot's content, and `{{md.slot.jobs}}` the `jobs` slot's. Each slot's content is blocks, paragraphs and lists, so put it on a line of its own.

The body decides where each slot goes. The work day places `jobs` under a heading of its own, and `bring` in a shaded box at the end. A slot can be placed anywhere a block can: in a `box`, in an `if`, even twice.

A slot the use doesn't give is empty, and nothing shows where it is placed. What the body puts around it still does: a work day without `:: #jobs` would still show the heading *On the list*.

## Filling slots

A use writes the default slot first, straight after the opener, then each named slot from its slot line to the next one, or to the closer:

````md
---
title: Elm Street Garden
---

```compmd component=notice
params:
  title: string
slots: [default, footer]
---
# {{title}}

{{md.slot}}

{{md.slot.footer}}
```

::: notice {title="Plots for next year"}

Sign up by November 30.

:: #footer

Plots go to last year's gardeners first.

::: /notice
````

- The default slot has no slot line, so it always comes first.
- The named slots come in any order: the work day gives `bring` before `jobs`, and the page shows them where the body places them.
- A slot line starts at the container's own level, not indented under a list item, which would put it inside the item (chapter 5).

What can go wrong:

| Written | Diagnostic |
| --- | --- |
| `:: #extra`, a slot the component doesn't declare | `F006`, which says the component has no slot `extra` |
| `:: #jobs` twice in one use | `F007`, and the second one's content joins the first |
| a slot line outside any container | `F015`, and the line is ignored |

## What slot content sees

The content of a slot is yours, not the component's: it is written in your document, and reads as it would anywhere in it. `{{md.doc.title}}` in a slot is your title, an `each`'s name around the use can be used inside it, and so can any component your document can name. The component's params are the component's: `{{hours}}` in a slot is a diagnostic (`F014`).

Headings in a slot keep the levels you wrote. Only the body's own headings follow where the component is used (chapter 12), since the body's writer can't know where it will go, and you do.

## Slots and their names

A slot's name is a word, such as `jobs` or `footer`, or words joined by `-`, as params' are: `:: #what-to-bring`. In the body, a path's steps can't hold a `-`, so it is written with `_`: `{{md.slot.what_to_bring}}` places it. Two slots of one component whose names differ only in `-` and `_` are `F013`, since their paths would be the same.

The built-in containers have slots too: `if` has `else`, and `each` has `before` and `empty` (chapter 5). They are filled the same way.

## A component with slots, as a leaf

A component with slots can still be used as a leaf: `:: notice[No news this month] {title="From the committee"}`. Its slots are then empty, and the page shows the box with its title. That's useful when the body has something to show without them.

Next: [template slots](14-template-slots.md), for a slot that shows once for each item of a list.
