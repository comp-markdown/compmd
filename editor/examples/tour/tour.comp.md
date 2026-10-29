---
title: Learn CompMD
author: {name: Ada Byron, role: editor}
released: "2026-10-29"
price: 12
tags: [markdown, components, data]
reading: {title: Piranesi, page: 120, pages: 272}
ideas:
  - {from: document, to: frontmatter}
  - {from: document, to: Markdown}
  - {from: document, to: components}
  - {from: frontmatter, to: data}
  - {from: frontmatter, to: inputs}
  - {from: components, to: params}
  - {from: components, to: slots}
  - {from: components, to: definitions}
  - {from: components, to: layout}
  - {from: slots, to: templates}
  - {from: definitions, to: JavaScript}
  - {from: data, to: expressions}
  - {from: inputs, to: expressions}
  - {from: expressions, to: params}
md:
  version: 1
  syntax: [wiki-links]
  data:
    - from: ./books.csv
      as: books
      schema: {title: string, author: string, year: number, pages: number, read: boolean}
  inputs:
    since: number = 1970
    unread: boolean = false
---

# {{md.doc.title}}

CompMD is Markdown with components: boxes and cards, values from data, lists that repeat, inputs the reader changes, layouts that follow the width of the page. This tutorial takes them one at a time. The text on the left is the document; the page on the right is what it renders, as you type.

Each lesson ends with **Try it**: a change to make in the text, and what to watch for on the page. Nothing you do here is saved, and the menu at the top brings the original back.

## 1. A document

A CompMD document is a Markdown file that starts with *frontmatter*: YAML between two `---` lines. CompMD's own settings go under the `md` key. Every other key is yours, and the document reads it as `md.doc.<key>`.

The heading above is written `# {{md.doc.title}}`: the `title` key, inserted where `{{…}}` stands. This document is by {{md.doc.author}}, the `author` key, which is a record: a record with a `name` reads as its name.

> [!TIP]
> **Try it.** Change [`title: Learn CompMD`](#:~:text=title:%20Learn%20CompMD) in the frontmatter, and watch the heading follow. (A link like this one selects what it names in the text.) Then open the **Data** tab beside the page: it shows every key the frontmatter gives the page.

## 2. The Markdown you know

Everything Markdown does still works, as CommonMark and GitHub write it: **bold**, *italic*, ~~struck~~, `code`, [links](https://commonmark.org), autolinks such as https://commonmark.org, and footnotes.[^plain]

[^plain]: Footnotes gather at the end of the page.

- Lists, tight or loose
- [x] Task lists
- [ ] with their boxes

| Tables | with | columns |
| --- | :---: | ---: |
| left | center | right |

> [!NOTE]
> Alerts come in five kinds: note, tip, important, warning and caution.

Math is written in TeX and typeset on the page: inline between single dollar signs, $e^{i\pi} + 1 = 0$, or as a block between lines of two, as below. Look at the text to see the TeX behind each; a renderer that doesn't typeset math shows that TeX instead.

$$
\sum_{n=1}^{\infty} \frac{1}{n^2} = \frac{\pi^2}{6}
$$

Wiki links are an option a document turns on, as this one does with `syntax: [wiki-links]`: [[Getting started]], or [[Getting started|with a label]]. Where a wiki link leads is the host's to say, so here they stay text.

> [!TIP]
> **Try it.** Open the **Plain Markdown** tab. It shows this page as a renderer that knows nothing of CompMD shows it: every component becomes readable text, so a CompMD file still reads well on GitHub or in any Markdown viewer.

## 3. Containers

A *container* wraps a body in a component. It opens with three colons and the component's name, and closes with the same name after a slash:

::: box {padding=m border=subtle border-radius=m}
This is a `box`, the simplest component: a box around its body. Anything Markdown holds goes in it: *text*, lists, other containers.

::: box {padding=s background=subtle border-radius=s}
A box inside a box. Each closer names its container, so it's always clear which one ends.
::: /box
::: /box

Every component line stands on a line of its own, with a blank line before and after it.

> [!TIP]
> **Try it.** Delete the [last `::: /box` line](#:~:text=:::%20/box-,:::%20/box). The page still renders, and a diagnostic under the text says which container was never closed, and clicking its place selects the line it means. Put the closer back, or press undo.

## 4. Params

What's in the braces are the component's *params*. They read like HTML attributes, with more kinds of values:

| You write | It is |
| --- | --- |
| `title="Setup"` | text, in quotes; `title="Step {{n}}"` inserts a value |
| `gap=m` | a bare word or number |
| `open` | a flag: `true` |
| `count=(size(rows))` | an expression, in parentheses (lesson 6) |
| `cols=["a", "b"]` | JSON |
| `.wide`, `#intro` | a class, an id |
| `md:gap=l` | a param from a width up (lesson 12) |

Every block component takes the same *common params* for its box: `padding`, `border`, `border-radius`, `background`, `width`, `max-width` and more. Their values are tokens (`none`, `s`, `m`, `l` for sizes; `subtle`, `strong`, `accent` for colors) or CSS values such as `1rem` or `#e9f3ff`:

::: box {#params .demo padding=l border=strong border-radius=l background=accent max-width=28rem}
A box with an id, a class, a large padding, a strong border, large corners, the accent background, and a width of at most 28rem.
::: /box

A long param list goes on to the next line, which starts `::: +`:

::: box {padding=m border=subtle}
::: + border-radius=full background=subtle
A pill.
::: /box

> [!TIP]
> **Try it.** Change [`border=strong`](#:~:text=border=strong) to `border=strng`. The box loses its border, and the diagnostic says what `border` takes, with a button that writes `strong` back.

## 5. Values

`{{path}}` inserts a value from the frontmatter, the data or the inputs: this page was released on {{md.doc.released}} and costs {{md.doc.price}} dollars. A path is names and dots, nothing more.

Anything computed or formatted comes from the inline component `value`, whose `of` param is an expression. What's in its brackets is its *fallback*, the text a plain renderer shows instead, so write what it usually says:

- Released :value[October 29, 2026]{of=(md.date(md.doc.released, "long"))}.
- Costs :value[$12.00]{of=(md.currency(md.doc.price, "USD"))}, or :value[$120.00]{of=(md.currency(md.doc.price * 10, "USD"))} for ten.
- Tagged :value[markdown, components, and data]{of=(md.list(md.doc.tags))}.
- :value[3 tags]{of=(string(size(md.doc.tags)) + " " + md.plural(size(md.doc.tags), "tag", "tags"))}.

To write braces that aren't a value, start them with a backslash: \{{like this}}.

> [!TIP]
> **Try it.** Change [`price: 12`](#:~:text=price:%2012) to `price: 9.5`, and watch every price follow. Then change [`"long"`](#:~:text=%22long%22) to `"full"` in the date.

## 6. Expressions

An expression in parentheses can compute any param. Expressions are [CEL](https://cel.dev), a small language with no loops and no side effects: `==`, `&&`, `? :`, `size(…)`, and methods on lists such as `filter`, `map` and `exists`.

::: box {padding=m border-radius=m background=(md.doc.price > 10 ? "accent" : "subtle")}
This box has the accent background while the price is over 10, and the subtle one otherwise: `background=(md.doc.price > 10 ? "accent" : "subtle")`.
::: /box

On a list, an expression counts, filters and adds: the reading list has :value[6 books]{of=(string(size(md.data.books)) + " books")}, :value[3 of them read]{of=(string(size(md.data.books.filter(b, b.read))) + " of them read")}, :value[1,719 pages]{of=(md.number(md.sum(md.data.books.map(b, b.pages)), 0) + " pages")} in all.

> [!TIP]
> **Try it.** Change [`price: 12`](#:~:text=price:%2012) to `price: 8`: the box above turns subtle. An expression with a mistake, say `md.doc.prise`, gives a diagnostic that points into the expression.

## 7. Showing and repeating

Two built-in containers decide what shows. `if` shows its body when `when` is true, and its `else` slot otherwise. A *slot* starts with a `:: #name` line:

::: if {when=(md.data.books.exists(b, !b.read))}
There are books left to read.

:: #else
Every book is read.
::: /if

`each` repeats its body once per item of a list, with the item under the name `as` gives it. With `join=list`, the items make one list; `before` shows above it when there are items, and `empty` when there are none:

::: each {in=(md.sortBy(md.data.books, "year")) as=b join=list}
1. *{{b.title}}*, {{b.author}} ({{b.year}})

:: #before
**The reading list, oldest first:**

:: #empty
The reading list is empty.
::: /each

The ordered list numbers on from one item to the next, and inside `each`, `md.loop` knows where it is: `md.loop.number`, `md.loop.first`, `md.loop.last`. With `join=inline`, the items make one paragraph instead, joined by `sep` and, before the last, `last`. Here, the authors:

::: each {in=(md.unique(md.data.books.map(b, b.author))) as=a join=inline sep=", " last=" and "}
{{a}}
::: /each

> [!TIP]
> **Try it.** Change [`md.sortBy(md.data.books, "year")`](#:~:text=md.sortBy(md.data.books%2C%20%22year%22)) to `md.data.books.filter(b, b.pages > 300)`: only the two books over 300 pages stay, numbered 1 and 2.

## 8. Your own components

A component of your own is a fence: `compmd component=<name>`, the params that style its box, then a *signature* (its params with their types, and its slots), a `---` line, and the body. A definition doesn't show on the page, so look for [`note`](#:~:text=compmd%20component=note) in the text on the left.

```compmd component=note {padding=m border=subtle border-radius=m background=(kind == "warning" ? "subtle" : "none")}
params:
  kind: tip | warning = tip
  title: string?
slots: [default, footer]
---
**{{title}}**

{{md.slot}}

{{md.slot.footer}}
```

In the body, `{{title}}` is the param, `{{md.slot}}` the body the caller writes, and `{{md.slot.footer}}` the `footer` slot. Defined anywhere in the document, it's used anywhere:

::: note {title="Before you start" kind=warning}
Back up your settings.

:: #footer
*It takes a minute.*
::: /note

Without a body, it's a *leaf*: two colons, and a fallback in brackets.

:: note[A note without a body] {title="A leaf"}

A component whose body is one paragraph can also be *inline*, inside a sentence, as [`tag`](#:~:text=compmd%20component=tag), defined in the text, is:

```compmd component=tag {padding="1px 8px" border-radius=full background=subtle}
params:
  label: string
---
{{label}}
```

This sentence has a :tag[draft]{label="draft"} in it, and a :tag[second one]{label="second one"}.

> [!TIP]
> **Try it.** Add `danger` to [`kind: tip | warning = tip`](#:~:text=kind:%20tip%20%7C%20warning%20=%20tip), then write `kind=danger` on [the note](#:~:text=:::%20note%20%7Btitle=%22Before%20you%20start%22). Write `kind=dangr` instead: the diagnostic lists the kinds the note takes. The **Components** tab lists every component this document defines and uses.

One more, as a taste of what's ahead. A component can also be code, and this one is JavaScript drawing with [d3](https://d3js.org): a map of the ideas in this tutorial, from the `ideas` key of the frontmatter. Drag a dot. Lesson 13, below, shows how such components work.

```compmd component=idea-map signature
params:
  links: array
```

```js component=idea-map
import * as d3 from "d3";

// The ideas as a force-directed graph, colored by how far each is from the document.
export default function mount(target, { params, fallback }) {
  const width = 640;
  const height = 320;
  const svg = d3.select(target).append("svg").attr("viewBox", [0, 0, width, height]).attr("width", "100%").attr("role", "img").attr("aria-label", fallback.text);
  let simulation;

  function draw(links) {
    simulation?.stop();
    svg.selectAll("*").remove();
    const nodes = [...new Set(links.flatMap((link) => [link.from, link.to]))].map((id) => ({ id }));
    const edges = links.map((link) => ({ source: link.from, target: link.to }));
    const depth = new Map([[nodes[0]?.id, 0]]);
    for (let changed = true; changed; ) {
      changed = false;
      for (const { source, target: to } of edges) {
        if (depth.has(source) && !depth.has(to)) {
          depth.set(to, depth.get(source) + 1);
          changed = true;
        }
      }
    }
    simulation = d3
      .forceSimulation(nodes)
      .force("link", d3.forceLink(edges).id((node) => node.id).distance(62))
      .force("charge", d3.forceManyBody().strength(-280))
      .force("center", d3.forceCenter(width / 2, height / 2))
      .force("collide", d3.forceCollide(30));
    const lines = svg.append("g").attr("stroke", "currentColor").attr("stroke-opacity", 0.25).selectAll("line").data(edges).join("line");
    const dots = svg
      .append("g")
      .selectAll("g")
      .data(nodes)
      .join("g")
      .style("cursor", "grab")
      .call(
        d3
          .drag()
          .on("start", (event, node) => {
            if (!event.active) simulation.alphaTarget(0.3).restart();
            node.fx = node.x;
            node.fy = node.y;
          })
          .on("drag", (event, node) => {
            node.fx = event.x;
            node.fy = event.y;
          })
          .on("end", (event, node) => {
            if (!event.active) simulation.alphaTarget(0);
            node.fx = null;
            node.fy = null;
          }),
      );
    dots.append("circle").attr("r", 8).attr("fill", (node) => `var(--viz-series-${String((depth.get(node.id) ?? 0) + 1)})`);
    dots.append("text").text((node) => node.id).attr("x", 12).attr("y", 4).attr("font-size", 13).attr("fill", "currentColor");
    simulation.on("tick", () => {
      for (const node of nodes) {
        node.x = Math.max(12, Math.min(width - 90, node.x));
        node.y = Math.max(12, Math.min(height - 12, node.y));
      }
      lines.attr("x1", (edge) => edge.source.x).attr("y1", (edge) => edge.source.y).attr("x2", (edge) => edge.target.x).attr("y2", (edge) => edge.target.y);
      dots.attr("transform", (node) => `translate(${node.x},${node.y})`);
    });
  }

  draw(params.links);
  return {
    update: (next) => draw(next.links),
    destroy: () => {
      simulation?.stop();
      svg.remove();
    },
  };
}
```

:: idea-map[A map of the ideas in this tutorial: the document has frontmatter, Markdown and components; components have params, slots, definitions and layout] {links=(md.doc.ideas)}

## 9. Templates

A component can repeat a template itself: a *template slot* renders once per element of a list param, under a name the caller picks. Here [`ranked`](#:~:text=compmd%20component=ranked), defined in the text, numbers whatever it's given:

```compmd component=ranked {display=flex flex-direction=column gap=s}
params:
  items: array
  as: name
slots:
  default: {each: items, as: as}
---
{{md.slot}}
```

::: ranked {items=(md.sortBy(md.data.books, "pages")) as=b}
**{{md.loop.number}}.** {{b.title}}, {{b.pages}} pages
::: /ranked

The caller decides what each element looks like; the component decides where it goes.

> [!TIP]
> **Try it.** In [`items=(md.sortBy(md.data.books, "pages")) as=b`](#:~:text=items=(md.sortBy(md.data.books%2C%20%22pages%22))%20as=b), change `"pages"` to `"title"`, or `as=b` to `as=book` and [the template's](#:~:text=**%7B%7Bmd.loop.number%7D%7D.**) `b.` to `book.`.

## 10. Data

Data comes in through the frontmatter's `data` key: a file, a URL or a source the host provides, read as `md.data.<as>`. This document reads `books.csv`, with a `schema` that types its columns, so `year` and `pages` are numbers and `read` is true or false. The tab above the text shows the file, read-only.

::: each {in=(md.data.books.filter(b, b.read)) as=b join=list}
- *{{b.title}}*: :value[304 pages]{of=(md.number(b.pages, 0) + " pages")}
::: /each

Read so far: :value[840 pages]{of=(md.number(md.sum(md.data.books.filter(b, b.read).map(b, b.pages)), 0) + " pages")}.

> [!TIP]
> **Try it.** Open `books.csv` in the tabs above the text, then come back to `tour.comp.md` and change `b.read` [in the list's filter](#:~:text=in=(md.data.books.filter(b%2C%20b.read))%20as=b) to `!b.read`.

## 11. Inputs

*Inputs* are values the reader changes. The frontmatter declares each with a type and a default, the page reads it as `md.input.<name>`, and an input component edits it:

:: number-input[Published since: 1970] {bind=since}

:: toggle-input[Unread only: no] {bind=unread}

::: each {in=(md.data.books.filter(b, b.year >= md.input.since && (!md.input.unread || !b.read))) as=b join=list}
- *{{b.title}}* ({{b.year}})

:: #empty
No book fits.
::: /each

The same inputs are in the bar above the tabs. There are five kinds: `date-input`, `number-input`, `text-input`, `select-input` for a list of choices, and `toggle-input`. **Inputs: plan a stay**, in the menu, uses them all.

> [!TIP]
> **Try it.** Set the year to 2000, or turn on *unread only*, and watch the list follow. Then change the year's default, [`since: number = 1970`](#:~:text=since:%20number%20=%201970), in the frontmatter.

## 12. Layout

Three built-ins lay out their blocks: `stack` one below the other, `row` side by side, `grid` in columns. Each top-level block inside is one item.

::: grid {min=9rem gap=s md:gap=m}
::: each {in=(md.data.books) as=b}
::: box {padding=s border=subtle border-radius=m}
**{{b.title}}**

{{b.author}}, {{b.year}}
::: /box
::: /each
::: /grid

`min=9rem` fits as many columns as there's room for, at least 9rem wide each. `md:gap=m` widens the gaps from the `md` width up. The widths are `sm` (480px), `md` (768px) and `lg` (1024px), measured on the box around the block, not the window.

::: box {padding=s background=subtle border-radius=s show=md}
You see this box when the page is at least 768px wide.
::: /box

::: box {padding=s background=subtle border-radius=s hide=md}
You see this box when the page is narrower than 768px.
::: /box

> [!TIP]
> **Try it.** Drag the handle between the text and the page to make the page narrow: the grid drops columns, and the two boxes above trade places.

## 13. Components in JavaScript

When CompMD isn't enough, a component can be code: JSX or TSX for React, Svelte, or plain JavaScript. The page runs it in a sandbox, with no network. In the text, a `jsx` fence holds [`progress`'s code](#:~:text=jsx%20component=progress): a React component, with its params as props. A signature fence types those params.

```compmd component=progress signature
params:
  value: number
  max: number
```

```jsx component=progress runtime=react@19
import { useEffect, useRef, useState } from "react";

// A ring that fills to `value` of `max`, easing from where it was to where it goes.
export default function Progress({ value, max, compmd }) {
  const share = Math.max(0, Math.min(1, value / max));
  const [shown, setShown] = useState(0);
  const from = useRef(0);
  useEffect(() => {
    const begin = from.current;
    const start = performance.now();
    const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
    let frame = 0;
    const step = (now) => {
      const t = still ? 1 : Math.min(1, (now - start) / 900);
      const next = begin + (share - begin) * (1 - (1 - t) ** 3);
      from.current = next;
      setShown(next);
      if (t < 1) frame = requestAnimationFrame(step);
    };
    frame = requestAnimationFrame(step);
    return () => cancelAnimationFrame(frame);
  }, [share]);
  const around = 2 * Math.PI * 40;
  return (
    <svg role="img" aria-label={compmd.fallback.text} viewBox="0 0 100 100" width="140" height="140">
      <circle cx="50" cy="50" r="40" fill="none" stroke="currentColor" strokeOpacity="0.12" strokeWidth="9" />
      <circle cx="50" cy="50" r="40" fill="none" stroke="var(--viz-series-1)" strokeWidth="9" strokeLinecap="round" strokeDasharray={around} strokeDashoffset={around * (1 - shown)} transform="rotate(-90 50 50)" />
      <text x="50" y="50" textAnchor="middle" dominantBaseline="central" fontSize="20" fontWeight="600" fill="currentColor">
        {Math.round(shown * 100)}%
      </text>
    </svg>
  );
}
```

Reading *{{md.doc.reading.title}}*, page {{md.doc.reading.page}} of {{md.doc.reading.pages}}:

:: progress[44% read: page 120 of 272] {value=(md.doc.reading.page) max=(md.doc.reading.pages)}

The idea map in lesson 8 is the plain JavaScript kind: its default export is a `mount` function that draws into the element it's given and returns `update` and `destroy`, and it imports `d3` as you'd write it anywhere. A component can import React, Svelte and d3, which the page provides.

> [!TIP]
> **Try it.** Change [`page: 120`](#:~:text=page:%20120) in the frontmatter to `page: 250`, and the ring sweeps on to the new value: the component keeps its state, and only its props change. Then change the code: [`stroke="var(--viz-series-1)"`](#:~:text=stroke=%22var(%2D%2Dviz%2Dseries%2D1)%22) to `stroke="var(--viz-series-2)"`, and the page runs the new code as you type.

**Your imagination is the limit.** A component is any code a browser runs, so a page can hold whatever you can build. Here is the reading list as a shelf, [drawn and animated with d3](#:~:text=js%20component=bookshelf): the books follow the inputs of lesson 11, and a click takes one off the shelf and opens it.

```compmd component=bookshelf signature
params:
  books: array
```

```js component=bookshelf
import * as d3 from "d3";

// The reading list as a shelf: the books rise in one after another, and a click takes one out and opens it.
export default function mount(target, { params, fallback }) {
  const still = matchMedia("(prefers-reduced-motion: reduce)").matches;
  const ms = (duration) => (still ? 0 : duration);
  // A book's color comes from its title, so it's the same on every shelf.
  const color = (title) => `var(--viz-series-${String((d3.sum([...title], (c) => c.codePointAt(0)) % 8) + 1)})`;
  const root = d3.select(target).append("div").attr("role", "group").attr("aria-label", fallback.text);
  const shelf = root
    .append("div")
    .style("display", "flex")
    .style("align-items", "flex-end")
    .style("gap", "6px")
    .style("height", "220px")
    .style("padding", "0 12px")
    .style("overflow", "hidden")
    .style("perspective", "900px")
    .style("border-bottom", "8px solid color-mix(in srgb, currentColor 22%, transparent)");
  const detail = root.append("p").attr("aria-live", "polite").style("min-height", "3em").style("margin", "12px 4px 0");
  let books = [];
  let open;

  const isOpen = (book) => book.title === open;
  const height = (book) => `${String(120 + Math.round((book.pages / 400) * 80))}px`;
  const width = (book) => (isOpen(book) ? "190px" : `${String(28 + Math.round(book.pages / 25))}px`);

  // Each book as it is now: open or on the shelf.
  function show() {
    shelf
      .selectAll("button")
      .attr("aria-expanded", (book) => String(isOpen(book)))
      .style("width", width)
      .style("transform", (book) => (isOpen(book) ? "translateY(-14px)" : "none"));
    shelf.selectAll(".cover").style("transform", (book) => (isOpen(book) ? "rotateY(-64deg)" : "none"));
    shelf.selectAll(".page").style("opacity", (book) => (isOpen(book) ? 1 : 0));
    const book = books.find(isOpen);
    detail.text(book ? `${book.title}, by ${book.author}: ${String(book.year)}, ${String(book.pages)} pages, ${book.read ? "read" : "not read yet"}.` : "Click a book to take it off the shelf.");
  }

  function draw(next) {
    books = next;
    if (!books.some(isOpen)) open = undefined;
    shelf
      .selectAll("button")
      .data(books, (book) => book.title)
      .join(
        (enter) => {
          const button = enter
            .append("button")
            .attr("type", "button")
            .attr("aria-label", (book) => book.title)
            .style("position", "relative")
            .style("flex", "none")
            .style("height", height)
            .style("padding", "0")
            .style("border", "1px solid color-mix(in srgb, currentColor 18%, transparent)")
            .style("border-radius", "3px 3px 1px 1px")
            .style("background", "var(--viz-surface)")
            .style("color", "inherit")
            .style("font", "inherit")
            .style("cursor", "pointer")
            .style("transform-style", "preserve-3d")
            .style("transition", still ? "none" : "width 500ms ease, transform 500ms ease")
            .style("opacity", 0)
            .style("translate", "0 80px")
            .on("click", (_event, book) => {
              open = isOpen(book) ? undefined : book.title;
              show();
            });
          button
            .append("span")
            .attr("class", "page")
            .style("position", "absolute")
            .style("inset", "0")
            .style("display", "flex")
            .style("flex-direction", "column")
            .style("justify-content", "flex-end")
            .style("gap", "4px")
            .style("padding", "12px 12px 12px 52px")
            .style("font-size", "12px")
            .style("text-align", "start")
            .style("transition", still ? "none" : "opacity 400ms ease 150ms")
            .selectAll("span")
            .data((book) => [book.title, book.author, `${String(book.year)}, ${String(book.pages)} pages`, book.read ? "Read" : "To read"])
            .join("span")
            .style("font-weight", (_line, i) => (i === 0 ? "700" : "400"))
            .style("font-style", (_line, i) => (i === 1 ? "italic" : "normal"))
            .text((line) => line);
          button
            .append("span")
            .attr("class", "cover")
            .style("position", "absolute")
            .style("inset", "0")
            .style("display", "flex")
            .style("align-items", "center")
            .style("justify-content", "center")
            .style("padding", "8px 0")
            .style("overflow", "hidden")
            .style("border-radius", "2px")
            .style("background", (book) => color(book.title))
            .style("color", "#fff")
            .style("font-size", "12px")
            .style("font-weight", "600")
            .style("white-space", "nowrap")
            .style("writing-mode", "sideways-lr")
            .style("transform-origin", "left center")
            .style("transition", still ? "none" : "transform 650ms cubic-bezier(0.2, 0.8, 0.2, 1)")
            .text((book) => book.title);
          button
            .transition()
            .delay((_book, i) => ms(i * 90))
            .duration(ms(650))
            .ease(d3.easeBackOut)
            .style("opacity", 1)
            .style("translate", "0 0px");
          return button;
        },
        (update) => update,
        (exit) => exit.transition().duration(ms(350)).style("opacity", 0).style("translate", "0 80px").remove(),
      );
    show();
  }

  draw(params.books);
  return {
    update: (next) => draw(next.books),
    destroy: () => root.remove(),
  };
}
```

:: bookshelf[The reading list as a shelf: click a book to see its author, year and pages] {books=(md.data.books.filter(b, b.year >= md.input.since && (!md.input.unread || !b.read)))}

A component written in code is used like any other, as often as you like, each time with its own params. And its params are expressions like any other, so they can use every input this document defines, just as before. This second shelf holds the books published *before* the year you picked in lesson 11, where the first holds those since:

:: bookshelf[The books published before the year picked above] {books=(md.data.books.filter(b, b.year < md.input.since))}

Components also nest, whatever they're written in. Here the `note` defined in lesson 8, written in CompMD, holds a shelf written in JavaScript, given only one author's books:

::: note {title="Ursula K. Le Guin on the shelf"}
:: bookshelf[The books by Ursula K. Le Guin] {books=(md.data.books.filter(b, b.author == "Ursula K. Le Guin"))}

:: #footer
*A note in CompMD, holding a component in JavaScript.*
::: /note

> [!TIP]
> **Try it.** Click *Solaris*, then click it again. Set the year above to 1975, and watch books move from one shelf to the other; turn on *unread only*, and the read books leave the first shelf. Then add `kind=warning` to [the note](#:~:text=:::%20note%20%7Btitle=%22Ursula).

## 14. Where next

The other documents in the menu go further, each on one side of CompMD:

- **Definitions and slots**, and **Expressions: a changelog**, for components built from components.
- **Data: a sales dashboard**, with charts in React and Svelte.
- **Inputs: plan a stay**, with every kind of input.
- **Layout: Kyoto in four days**, with grid areas and blocks placed by position.
- **Diagnostics and their fixes**, five mistakes and how the editor mends them.

The [specification](https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md) says everything precisely.
