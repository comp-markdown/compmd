---
md:
  export:
    pack: "@comp-markdown/sticky-note"
    version: "0.2.0"
    names: [sticky-note]
---

# @comp-markdown/sticky-note

`sticky-note` is a note pinned on the page: a yellow box with a red pin at its top, tilted a little, with its corner turned up and a shadow, for a reminder beside a walk's directions, a tip in a recipe, a correction in a newsletter.

```md
::: sticky-note {sm:inset-inline-end=1rem sm:inset-block-start=4rem}
Bring the tide table: the quay floods at high water.
::: /sticky-note
```

What you write in it is the note. Without a position it sits where it is written, in the flow. `inset-*` pins it in its nearest box (spec §9.2): `inset-inline-end=1rem inset-block-start=4rem` puts it 1rem from the box's end and 4rem from its top, over what is there; the box is as tall as what stays in its flow. Written from `sm` up, `sm:inset-…`, it is pinned where its box is wide and in the flow on a phone, where there is no room beside the text.

It is 14rem wide, never wider than its column, classic yellow with dark text in light and dark pages alike; a use changes its box as any box's, `{rotate=3deg background=subtle color=accent width=10rem}`. The note leaves its bottom end corner out, so the page shows through there (a gradient, spec §9.4), and the corner turned up over it is a box of its own, as the pin is: they carry no text, so a screen reader reads the note alone. The corner follows the text's direction, at the bottom end. A use that changes `background` paints the whole note, corner included. It prints as it shows. In a plain Markdown reader, a note is its text.

```compmd component=sticky-note {padding=m padding-block-start=l background="linear-gradient(315deg, transparent 1.4rem, #fff176 1.4rem)" color="#3b3000" shadow=m rotate=-2deg width=14rem max-width=100%}
slots: [default]
---
::: box {inset-block-start=0.35rem inset-inline-start=46% width=0.9rem height=0.9rem border-radius=full background="#c62828" shadow=s}
::: /box

{{md.slot}}

::: box {inset-block-end=0 inset-inline-end=0 width=1.98rem height=1.98rem background="linear-gradient(315deg, transparent 50%, #e0c93c 50%)"}
::: /box
```
