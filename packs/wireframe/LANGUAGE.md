# The wireframe language

**Status: accepted (2026-11-17).**

The language of a ```` ```wireframe ```` block, which this pack draws
([its design](https://github.com/comp-markdown/compmd/blob/main/docs/wireframe-pack.md)): a screen as an
indented outline, one part per line. It is Wireloom's syntax, as Wireloom's documentation and examples show it on
2026-11-17 ([StardockCorp/Wireloom](https://github.com/StardockCorp/Wireloom), MIT), written down here as a grammar of
our own, with the parts the pack draws first, how each is drawn in HTML and CSS, and its errors.

````md
```wireframe {alt="The sign-in screen: email, password, and a Sign in button"}
window "Sign in":
  header:
    text "Welcome back" bold
  panel:
    input placeholder="Email" id=email
    input placeholder="Password" type=password
    row justify=between:
      checkbox "Remember me"
      button "Sign in" primary
annotation "Shows an error under the field that failed" target="email" position=right
```
````

## Decisions

1. **Wireloom's syntax, and a superset of it.** Every source Wireloom's documentation accepts is a source here, and
   means the same; what this language adds is marked *(ours)* below, so a source that uses none of it is Wireloom's
   too. Where Wireloom's documentation doesn't say (escapes, the order of a line's arguments), this grammar decides,
   and says so.

2. **A source is an outline** (the grammar is under *Grammar*):
   - one `window` at the top level, its parts indented under it, and then any `annotation` lines, also at the top
     level;
   - a line that ends in `:` takes the lines indented below it as its children; a line without one is a leaf;
   - an indentation level is 2 or 4 spaces, the first indented line choosing which for the whole source; a tab, or a
     line indented by another amount, is an error;
   - a line starting with `#` is a comment; comments and blank lines change no nesting.

3. **A line is a part's name, then its arguments**: its positional arguments first (quoted strings, or a number for
   a `col`'s width), then its attributes (`key=value`) and its flags (bare words) in any order *(Wireloom's
   documentation lists attributes before flags; either order is read here)*. A value is a quoted string, a number with
   an optional unit (`340`, `50%`, `1fr`), a range (`0-100`), or a bare name (`password`). In a quoted string, `\n` is
   a line break, and *(ours)* `\"` a quote and `\\` a backslash. Every part takes `id="…"`, which annotations point
   at.

4. **The parts drawn first**, the ones a wireframe of a web or an application screen needs:

   | Part | Children | Positional | Attributes and flags |
   |---|---|---|---|
   | `window` | any | a title | *(ours)* `device=phone\|tablet\|browser\|desktop`, the frame it is drawn in (`desktop` by default) |
   | `header`, `footer` | any | | `header`: `large` |
   | `panel` | any | | |
   | `section` | any | a title (required) | `badge="…"` |
   | `row` | any | | `align=left\|center\|right`, `justify=start\|between\|around\|end` |
   | `col` | any | a width (`240`, `30%`), or the flag `fill` (the default) | |
   | `grid` | `cell` only | | `cols=N`, `rows=M` (required) |
   | `cell` | any | a label | `row=N`, `col=M` (from 1) |
   | `tabs` | `tab` only | | |
   | `tab` | | a label | `active`, `badge="…"` |
   | `list` | `item` only | | |
   | `item` | | a text | `chevron` |
   | `tree` | `node` only | | |
   | `node` | `node` | a label | `collapsed`, `selected`, `icon="…"` |
   | `menubar` | `menu` only | | |
   | `menu` | `menuitem`, `separator` | a title (required) | |
   | `menuitem` | | a label | `shortcut="…"`, `disabled` |
   | `separator`, `divider` | | | |
   | `text` | | its text (required) | `bold`, `italic`, `muted`, `weight=light\|regular\|semibold\|bold`, `size=small\|regular\|large` |
   | `button` | | a label (empty for an icon alone) | `primary`, `disabled`, `icon="…"`, `badge="…"` |
   | `input` | | | `placeholder="…"`, `type=…` (`password` draws dots), `disabled` |
   | `combo` | | a label | `value="…"`, `options="…"`, `disabled` |
   | `checkbox` | | a label | `checked`, `disabled`, `label-right` |
   | `radio` | | a label | `selected`, `disabled`, `label-right`, `group="…"` |
   | `toggle` | | a label | `on`, `off`, `disabled`, `label-right` |
   | `progress` | | | `value=…`, `max=…`, `label="…"` |
   | `image` | | | `label="…"`, `width=…`, `height=…` |
   | `icon` | | | `name="…"` |
   | `spacer` | | | only in a `row` |
   | `annotation` | | its text (required) | `target="…"` and `position=left\|right\|top\|bottom` (required) |

   Wireloom's other parts are read and checked as its documentation says, and drawn as a box with their name and
   label until the pack draws them: the application-bar parts (`navbar`, `tabbar`, `sheet`, `backbutton`,
   `segmented`, `segment`, `breadcrumb`, `crumb`), the content parts (`kv`, `slider`, `chart`, `chip`, `avatar`,
   `spinner`, `status`), and those made for game screens (`slot`, `resourcebar`, `resource`, `stats`, `stat`, the
   `accent=` and `state=` values). A source that uses them stays valid.

5. **Drawn as HTML and CSS**, each part a small function:
   - containers are boxes: `row` a flex row (`justify`, `align` as `justify-content` and `align-items`), `col` a flex
     column of its width, `grid` a CSS grid of `cols` and `rows` with each `cell` at its place, `panel` a box with a
     dashed border, `section` a titled box;
   - `window` is the frame: a window with a title bar, or *(ours)* a phone's or a tablet's outline, or a browser's
     bar, as `device` says; it is as wide as its block, and as tall as its content;
   - controls are drawn as the controls they stand for, in greys: a `button` a bordered box, `primary` filled; an
     `input` a box with its placeholder muted; a `combo` a box with a chevron; `checkbox`, `radio` and `toggle` their
     marks; `tabs` a row of tabs over a line; `menubar` a bar of its menus' titles, each `menu` drawn open below its
     title; `tree` an indented list with chevrons;
   - `image` is a crossed box of its size; `icon` a small square with the icon's first letter, as Wireloom draws an
     icon it doesn't know (a set of drawn icons comes with the pack later);
   - `annotation` is a numbered note beside the window, on its `position`'s side, with a number at its target, the
     same number in both places; a note whose target no part has is an error *(Wireloom drops it silently)*;
   - the look is a sketch's: greys from the host's tokens, borders a little uneven, light and dark by the page's
     scheme (the host contract's `scheme`), a hand-written font when the page has one; it prints as it shows.
   - The block's `alt` is the drawing's text alternative, as for any code renderer (spec §7.7); the parts' texts are
     text in the HTML, read and selected as text.

6. **Errors**: a source that isn't valid draws nothing and fails with W118 (spec §7.4's `fail`), its message naming
   the line and column in the block (`line 4, column 5: ...`) and what to write instead, as Wireloom's do: a tab or an
   indentation that isn't the source's unit; an unknown part (with the nearest name), an argument a part doesn't take,
   a value outside its list; a required argument missing; a child its parent doesn't take, or a leaf with children;
   more than one `window`, or none; a part where its rules forbid it (`spacer` outside a `row`, `tab` outside `tabs`);
   an `annotation` without `target` or `position`, or whose target no part has.

## Grammar

ABNF ([RFC 5234](https://www.rfc-editor.org/rfc/rfc5234)), for one line at a time. Which lines are whose children is
set by their indentation (decision 2), not by the grammar.

```abnf
line         = blank / comment / part
blank        = *WSP
comment      = indent "#" *( VCHAR / WSP )
part         = indent name *( 1*SP argument ) [ *SP ":" ] *WSP
indent       = *SP                         ; a multiple of the source's unit, 2 or 4 spaces
name         = ALPHA *( ALPHA / DIGIT / "-" )
argument     = string / number / attribute / flag
                                           ; positionals (strings, numbers) before attributes and flags
attribute    = name "=" value
flag         = name
value        = string / number / range / word
string       = DQUOTE *( char / escape ) DQUOTE
char         = %x20-21 / %x23-5B / %x5D-10FFFF   ; any but a quote and a backslash
escape       = "\" ( "n" / DQUOTE / "\" )       ; `\"` and `\\` are ours
number       = 1*DIGIT [ "." 1*DIGIT ] [ unit ]
unit         = "px" / "%" / "fr"
range        = 1*DIGIT "-" 1*DIGIT
word         = ALPHA *( ALPHA / DIGIT / "-" / "_" )
```

## Examples

Each is a test of the parser and of the drawing.

**Settings, with tabs, checkboxes and radios:**

```wireframe
window "Settings":
  tabs:
    tab "General" active
    tab "Privacy"
    tab "Advanced"
  section "Notifications":
    toggle "Email me about replies" on
    checkbox "Weekly summary" checked
  section "Theme":
    radio "Light" group="theme"
    radio "Dark" group="theme" selected
    radio "Follow the system" group="theme"
  footer:
    row justify=end:
      button "Cancel"
      button "Save" primary
```

**A file manager, with a menu bar, a tree and a grid:**

```wireframe
window "Files":
  menubar:
    menu "File":
      menuitem "New folder" shortcut="⌘N"
      separator
      menuitem "Close" shortcut="⌘W"
    menu "Edit"
  row:
    col 220:
      tree:
        node "Documents" selected:
          node "Reports"
          node "Drafts" collapsed
        node "Pictures"
    col fill:
      grid cols=3 rows=2:
        cell "report.pdf" row=1 col=1
        cell "notes.txt" row=1 col=2
        cell "photo.png" row=1 col=3
```

**A phone screen** *(ours: `device`)*, with an annotation:

```wireframe
window "Order" device=phone:
  header:
    text "Your basket" bold
  list:
    item "Coffee beans, 1 kg" chevron
    item "Filter papers" chevron
  progress value=2 max=3 label="Step 2 of 3"
  button "Pay now" primary id=pay
annotation "Disabled until an address is chosen" target="pay" position=bottom
```

**Errors:** `window "A":` then a line indented by a tab (`line 2, column 1: a tab in the indentation: indent with 2 or 4
spaces`); `butn "Go"` (`line 3, column 3: no part \`butn\`: did you mean \`button\`?`); `spacer` in a `col` (`spacer is
only in a row`).

## Where ours differs from Wireloom

- *(ours)* `window`'s `device`, and the `\"` and `\\` escapes: a source that uses them isn't Wireloom's.
- A line's attributes and flags in any order after its positionals.
- An annotation whose target no part has is an error, not dropped.
- The parts of decision 4's second paragraph are read and checked, but drawn as named boxes until the pack draws them.
- Icons are drawn as their first letter until the pack has a set of its own; Wireloom's icon names are game icons.
- It is drawn as HTML and CSS, not SVG: its layout follows the width it is given, and its text is text.
