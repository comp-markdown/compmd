---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

Each example of `wireframe` shows its source, then what it draws.

# A screen on a phone

A `wireframe` code block sketches a screen; `device=phone` frames it as one, and an annotation points at a part by its `id`.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

```wireframe {alt="The basket on a phone: two items, step 2 of 3, and a Pay now button that stays disabled until an address is chosen"}
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
````

# A settings window

Tabs, sections, toggles, radio buttons, and a footer whose buttons sit at its end.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

```wireframe {alt="The settings window: General, Privacy and Advanced tabs; notification toggles; a theme chosen with radio buttons; Cancel and Save"}
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
````

# A sketch from the document's data

As a leaf, its `code` computed: the step tabs come from the document's list, so the sketch follows when a step is added.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
tabs: [Basket, Address, Payment]
---

:: wireframe[The checkout's steps as tabs: Basket, Address, Payment] {code=("window \"Checkout\" device=browser:\n  tabs:\n" + md.doc.tabs.map(t, "    tab \"" + t + "\"").join("\n"))}
````

# In a narrow box

A sketch is laid out in its box as a page is, so a narrower box gives a narrower window.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

:: wireframe[A narrow sign-in window: an email field and a button] {code=("window \"Sign in\":\n  input placeholder=\"Email\"\n  button \"Continue\" primary") width=18rem}
````

# Every part

What the pack draws: menus, a tree, a grid, fields, an image, an icon, a chip and notes.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

```wireframe {alt="A file manager: a menu bar, a folder tree and a grid of files"}
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

```wireframe {alt="A sign-up form on a tablet: fields, a country, an avatar placeholder and a note on the email field"}
window "Sign up" device=tablet:
  panel:
    text "Create an account" size=large weight=semibold
    text "It takes a minute." muted italic
    input placeholder="Email" id=email
    input placeholder="Password" type=password
    combo "Country" options="Spain, Japan, Brazil"
    row align=center:
      image width=64 height=64 label="Photo"
      icon name="camera"
      spacer
      button "Upload" icon=upload badge="1"
    divider
    checkbox "I agree to the terms" label-right
    button "Create account" primary disabled
  chip "Beta"
annotation "Checked while you type" target="email" position=right
annotation "Shown before the form is sent" target="email" position=left
```
````

# A source with a mistake

Its code shows instead, and the host reports where the mistake is.

````compmd {render=both}
---
md:
  import:
    - from: "@comp-markdown/wireframe@0"
      names: [wireframe]
---

```wireframe {alt="A screen with a mistake"}
window "Broken":
  butn "Go"
```
````
