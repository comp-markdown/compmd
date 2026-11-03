# 14. Signatures and params

A component's signature says what a document may give it: its params, their types and defaults, and its slots. Hosts check every use against it before any code runs, so a mistake shows as a diagnostic on the line that made it, and the component gets only values of the types it declared. This chapter builds a pack of two components, a rating the reader sets and lanes of records, and goes through every type, slots, template slots, inputs and how a use is checked.

## The signature fence

An exported component written in code must have a signature (F029 otherwise), in a fence of the language `compmd` with `component=<name>` and the flag `signature`, in the pack's document. It holds YAML: `params`, a mapping of each param's name to its declaration, and `slots`, the slots it takes. A pack's document may declare several components, each imported from its own built module:

````md snippet=rating-pack/index.comp.md
---
md:
  import:
    - from: ./dist/rating.js
      names: [rating]
    - from: ./dist/lanes.js
      names: [lanes]
  export:
    pack: "@you/rating"
    version: "0.1.0"
    names: [rating, lanes]
---

# @you/rating

`rating`, a leaf the reader sets, bound to a number input: `:: rating[3 of 5 stars] {value=stars}`.

```compmd component=rating signature
params:
  value: input<number>
  max: number = 5
  size: small | large = small
  color: color?
```

`lanes`, a container whose default slot is a template, once per record, in the lane of its `by` field.

```compmd component=lanes signature
params:
  rows: array
  by: string
  as: name
slots:
  default: {each: rows, as: as}
```
````

A signature fence takes no param list (F036); a component's box is styled by the document that uses it. `renders`, for code renderers, is [chapter 16](16-code-renderers.md)'s.

The build makes one module for each `./dist/<name>.js` the document imports, `dist/rating.js` from `src/rating.ts` and `dist/lanes.js` from `src/lanes.ts`. The rest of the pack is the template's, with its own name:

```json snippet=rating-pack/package.json
{
  "name": "@you/rating",
  "version": "0.1.0",
  "description": "A CompMD pack: a rating the reader sets, and lanes of records grouped by a field.",
  "keywords": ["compmd", "compmd-pack"],
  "license": "MIT",
  "type": "module",
  "compmd": "./index.comp.md",
  "files": ["dist", "index.comp.md", "README.md"],
  "scripts": {
    "build": "compmd-pack build",
    "dev": "compmd-pack dev",
    "test": "compmd-pack test",
    "lint": "tsc --noEmit",
    "prepublishOnly": "compmd-pack build && compmd-pack test"
  },
  "devDependencies": {
    "@compmd/pack": "0.3.0",
    "happy-dom": "21.0.3",
    "typescript": "6.0.3",
    "vitest": "5.0.5"
  }
}
```

```json snippet=rating-pack/tsconfig.json
{ "extends": "@compmd/pack/tsconfig.json", "include": ["src"] }
```

## Declarations

A declaration's short form is a YAML string, its type, then `?` when it is optional, then ` = ` and a default:

```text
max: number = 5
size: small | large = small
color: color?
```

The long form is a mapping, for a default the short form can't write (one with `: ` in it, such as a JSON object, isn't a valid YAML string) or a description for editors:

```text
meta:
  type: object
  default: {source: survey, year: 2026}
  doc: Where the numbers come from
```

A type that isn't one of those below, such as a misspelt `numbr`, is F037 at the type's name. The param is kept, but takes any value unchecked, so the one mistake gives one diagnostic, not one at each use.

## The types

| Type | A document writes | The component gets |
|---|---|---|
| `string` | `title="Setup"`, or bare, `title=Setup` | the text |
| `number` | bare: `max=5`, `max=-0.5` | a number |
| `boolean` | `open=true`, `open=false`, or the flag `open` | `true` or `false` |
| `date` | `due=2026-10-21` | the text `2026-10-21` |
| `array` | JSON on one line: `values=[3, 5, 8]` | an array |
| `object` | JSON on one line: `meta={"year": 2026}` | an object |
| an enum, `small \| large` | one of its members, bare or quoted: `size=large` | the member, as text |
| `length` | `px`, `rem`, `em` or `%`, or `0`: `width=12rem` | the text |
| `space` | `none`, `s`, `m`, `l`, or a length | the text |
| `color` | a CSS color: `#b8860b`, `teal`, `oklch(0.7 0.1 200)` | the text |
| `tracks` | grid tracks: `"1fr 2fr"`, `"repeat(auto-fill, minmax(12rem, 1fr))"` | the text |
| `areas` | quoted rows of area names, all as long: `"'head head' 'side main'"` | the text |
| `surface` | `none`, `subtle`, `strong`, `accent`, or a CSS color | the text |
| `border` | `none`, `subtle`, `strong`, or a CSS border: `"1px solid #d0d7de"` | the text |
| `radius` | `none`, `s`, `m`, `l`, `full`, or a length | the text |
| `name` | a bare identifier, never read as an expression: `as=t` | the text |
| `input<type>` | the name of an input the document declares: `value=stars` | the input's value |

- **Any type but `name` and `input<…>` takes an expression**, `rows=(md.doc.tasks)`, checked by its type when the document is validated (F024 otherwise), and evaluated before the component sees it.
- **A number, a boolean and a name are written bare.** `max="5"` is F009: a quoted value is a string. A string may be written bare, and a quoted one may interpolate: `title="Step {{n}}"`.
- **Enum members are names**, written without quotes in the signature; a member with spaces needs the whole declaration quoted (`status: '"to do" | done = done'`). An enum has at least two members: a single name is a type name, so `size: small` is F037.
- **The CSS types are closed.** A value outside its grammar is F009, so no param passes raw CSS through. The component gets the text as written, and a token (`m`, `subtle`) is its to read: the host gives a component's own params no meaning.
- There are no other types: a list is an `array` and a record an `object`, whose contents the signature doesn't check. Slots are declared under `slots`, not as params.

## Defaults and required params

- A param with a default is optional: the component gets the default when the document doesn't give one. `max` is `5` in every use that doesn't say.
- A param whose type ends in `?` is optional with no value: when the document doesn't give it, it isn't in `params` at all, so its TypeScript type is optional (`color?: string`).
- Any other param is required: a use without it is F010, and renders its fallback, so the component never sees a use that lacks it.

The component that reads them:

```ts snippet=rating-pack/src/rating.ts
import { component } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them: `color` is optional, so it may be missing. */
export interface Params {
  value: number;
  max: number;
  size: "small" | "large";
  color?: string;
}

export default component<Params>((target, { params, fallback, set }) => {
  const stars = document.createElement("span");
  stars.setAttribute("role", "group");
  stars.setAttribute("aria-label", fallback.text);
  target.append(stars);
  const draw = ({ value, max, size, color }: Params) => {
    stars.style.fontSize = size === "large" ? "1.6em" : "1em";
    stars.style.color = color ?? "inherit";
    stars.replaceChildren(
      ...Array.from({ length: max }, (_, i) => {
        const star = document.createElement("button");
        star.type = "button";
        star.textContent = i < value ? "★" : "☆";
        star.setAttribute("aria-label", `${String(i + 1)} of ${String(max)}`);
        // The reader's choice goes to the input `value` is bound to; the host renders again with it.
        star.addEventListener("click", () => {
          set("value", i + 1);
        });
        return star;
      }),
    );
  };
  draw(params);
  return {
    update: draw,
    destroy: () => {
      stars.remove();
    },
  };
});
```

The signature checks types and an enum's members, never what values mean together, or a number's range. `max=-3` is a number, so the component is the one to decide what a negative count draws: here, no stars. Params come from documents anyone writes, so treat them as untrusted input. Never put one in HTML as markup: `rating` writes text with `textContent`.

## Slots

`slots: [default, footer]` makes the component a container that takes a body, its content before the first `:: #footer` line the default slot, and what follows the footer. A component with no `slots` key takes no body: a use with one is F018, and the body follows the component as ordinary content. A slot the signature doesn't declare is F006, and its content joins the default slot.

The component gets each slot in `context.slots`, by name, and again as `update`'s second argument: a list of items, one per top-level block of the slot, each `{ element, component? }`. The host rendered each block into its `element`, a box of its own, and keeps it live, including the components inside it, whatever framework the parent uses. The component only places the elements in its own DOM: never copy their HTML. When the block is a component, `component` holds its name and params, so a tabs component can read its tabs' labels. Every slot is rendered up front, even what the component hides.

## Template slots

A template slot is rendered once per element of an array param, with a name the author picks for the element: a board that puts each card in its lane, a calendar that puts each event on its day. The slot's options name the array param (`each`) and a `name` param (`as`) of the same component:

```text
slots:
  default: {each: rows, as: as}
```

The component gets the slot as a list of instances instead of items, one per element: `{ value, items }`, the element and the blocks its template rendered.

```ts snippet=rating-pack/src/lanes.ts
import { component, type Slots } from "@compmd/pack/component";

/** The params, as the signature in `index.comp.md` declares them. `as` is the author's name for a record. */
export interface Params {
  rows: Record<string, unknown>[];
  by: string;
  as: string;
}

/** One instance of the template slot: its record, and the blocks the template rendered for it. */
interface Instance {
  value: unknown;
  items: { element: Element }[];
}

export default component<Params>((target, { params, slots }) => {
  const board = document.createElement("div");
  board.style.cssText = "display: flex; gap: 1rem; align-items: start;";
  target.append(board);
  const draw = ({ by }: Params, { default: instances = [] }: Slots) => {
    const lanes = new Map<string, HTMLElement>();
    for (const { value, items } of instances as Instance[]) {
      const field = (value as Record<string, unknown> | null)?.[by];
      const name = typeof field === "string" ? field : "";
      let lane = lanes.get(name);
      if (!lane) {
        lane = document.createElement("section");
        lane.append(Object.assign(document.createElement("h3"), { textContent: name }));
        lanes.set(name, lane);
      }
      // The renderer made these elements, and keeps what is in them live: the component only places them.
      lane.append(...items.map(({ element }) => element));
    }
    board.replaceChildren(...lanes.values());
  };
  draw(params, slots);
  return {
    update: draw,
    destroy: () => {
      board.remove();
    },
  };
});
```

`rows` is typed `Record<string, unknown>[]` for reading, but an `array` may hold anything a document gives, so `lanes` checks each field before it uses it. Its test builds the instances a renderer would give:

```ts snippet=rating-pack/src/lanes.test.ts
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./lanes.ts";

/** A template slot's instance, as the renderer gives it: a record, and an element for what its template rendered. */
const instance = (title: string, status: string) => ({ value: { title, status }, items: [{ element: Object.assign(document.createElement("p"), { textContent: title }) }] });

describe("lanes", () => {
  it("puts each record's rendered template in the lane of its field, lanes in the order they first appear", () => {
    const rows = [
      { title: "Write", status: "doing" },
      { title: "Plan", status: "done" },
      { title: "Review", status: "doing" },
    ];
    const slots = { default: rows.map(({ title, status }) => instance(title, status)) };
    const shown = mountForTest(mount, { params: { rows, by: "status", as: "t" }, slots });
    const lanes = [...shown.target.querySelectorAll("section")].map((lane) => lane.textContent);
    expect(lanes).toEqual(["doingWriteReview", "donePlan"]);
    shown.destroy();
  });
});
```

## Inputs and set

Components have one output: changing an input the document binds to them. A param declared `input<number>` is given the name of an input the document declares, and the component gets that input's current value. `set("value", 4)` changes the input; the host processes the document again with the new value and updates every component that depends on it. A document using both components:

```text
---
md:
  import:
    - from: "@you/rating@0"
      names: [rating, lanes]
  inputs:
    stars: number = 3
tasks:
  - {title: Write the outline, status: doing}
  - {title: Ask for a review, status: waiting}
  - {title: Draft the first part, status: doing}
---

:: rating[3 of 5 stars] {value=stars size=large}

::: lanes {rows=(md.doc.tasks) by=status as=t}
**{{t.title}}**
::: /lanes
```

`rating` gets `{ value: 3, size: "large", max: 5 }`, then 4 once the reader chooses the fourth star. `lanes` gets three instances, each rendered from `**{{t.title}}**` with `t` its record: `t` is the author's name, resolved in the document, so the component never sees the template, only what it rendered.

- **The binding is checked:** a name no input under `md.inputs` declares, or an input of another type, is F030. `input<enum>` binds an input of any enum, and `input<small | large>` only one with exactly those members, in that order.
- **`set` is refused** for a param that isn't bound to an input, and during `update`: in `@compmd/render` it throws. Call it from what the reader does, as `rating` does from a click.
- **A value of the wrong type isn't the component's to send.** `set` sends any value; the host gives it to the processor as the input's value, and the spec has the processor use the input's default instead of a value that doesn't match its type (spec [§8.7](https://compmd.hakanserce.com/spec/#87-inputs)).
- **An input a component can change can't choose what is loaded:** a data `from` that interpolates it is F031, so component code can't send data out through it.

The test names the params bound to an input, and reads what the component set:

```ts snippet=rating-pack/src/rating.test.ts
import { mountForTest } from "@compmd/pack/testing";
import { describe, expect, it } from "vitest";
import mount from "./rating.ts";

describe("rating", () => {
  it("draws its value of its max, and sets the bound input when a star is chosen", () => {
    const shown = mountForTest(mount, { params: { value: 3, max: 5, size: "small" }, fallback: "3 of 5 stars", inputs: ["value"] });
    const stars = () => [...shown.target.querySelectorAll("button")];
    expect(stars().map((star) => star.textContent).join("")).toBe("★★★☆☆");
    stars()[4]?.click();
    expect(shown.sets).toEqual([{ param: "value", value: 5 }]);
    shown.update({ value: 5, max: 5, size: "large", color: "#b8860b" });
    expect(stars().map((star) => star.textContent).join("")).toBe("★★★★★");
    shown.destroy();
  });
});
```

## Fallbacks

Every use has a fallback, which the component gets as `fallback`: a leaf's or an inline component's text in `[…]`, a container's slots as plain Markdown. It is what a plain renderer shows, what a host shows when the component can't render, and the component's text alternative, `fallback.text`. A leaf without one is W101: plain renderers show nothing, and the component has no text alternative. Write it as a sentence that says what the component shows: `3 of 5 stars`, not `rating`.

The fallback is in a document's hands, so don't rely on it for anything but text.

## How a use is checked

Hosts validate every use against its component's signature after resolving the document's imports and before loading data or running code (spec [§5.3](https://compmd.hakanserce.com/spec/#53-validation)). Each diagnostic points at the param, not the whole line, and the document still renders:

| Code | When | What renders |
|---|---|---|
| F008 | a param the signature doesn't declare, and isn't a common param, `class`, `id`, `lang` or `dir`; or a breakpoint a param doesn't take (a component's own params take none) | the param is ignored |
| F009 | a value outside its type: `max=many`, `max="5"`, an enum's non-member, an expression where a `name` or an input is bound | the param is treated as absent: its default, or the fallback when it is required |
| F010 | a required param missing | the fallback |
| F024 | an expression whose type isn't the param's | as F009 |
| F030 | an `input<…>` param naming no declared input, or one of another type | the fallback |
| F006 | a slot the signature doesn't declare | its content joins the default slot |
| F018 | a body given to a component with no slots | the body follows it as ordinary content |

F008 and F009 suggest the nearest name or token when a value looks misspelt, with a fix an editor offers. One mistake gives one diagnostic: a required param whose value was refused isn't reported missing again. A document with a mistake in each use:

```text
---
md:
  import:
    - from: "@you/rating@0"
      names: [rating]
  inputs:
    stars: number = 3
    note: string = ""
---

:: rating[Stars] {value=stars colour=gold}

:: rating[Stars] {value=stars max=many}

:: rating[Stars] {value=stars size=huge}

:: rating[Stars] {max=4}

:: rating[Stars] {value=note}
```

gives F008 (`rating` has no param `colour`, so it is ignored), F009 twice (`max` takes a number, `size` takes `small` or `large`), F010 (`rating` needs `value`) and F030 (`value` binds a number, but `note` is a string). The first three uses render the component, with the bad params left out; the last two render `Stars`.

Next: [React, Svelte and libraries](15-react-svelte-and-libraries.md), rendering a framework inside `mount`, and what a pack bundles.
