# What a param takes

**Status: design, for review.**

A param value outside its type is F009, and today the message often says only "`padding` doesn't take this value". This design makes every F009 say what the param takes, and makes what a param takes a single definition, from which the check, the message, the suggestion and the renderer's tests all come. Adding a token, a type or a param then can't leave one of them behind.

## Decisions

1. **One definition per kind of value.** Each kind (`space`, `border`, `color`, `tracks`, …) is one entry in one table in `validate/values.rs`: its tokens, the CSS forms it also takes, and an example of those forms. Checking a value, describing the kind in a message and suggesting a token are three functions of that entry. No token list is written anywhere else in the processor.
2. **Every param gets its kind by an exhaustive `match`.** A declared type name (`TypeName`) and a common param (`Universal`) each map to a kind in a `match` with no `_` arm, so a new type name or common param doesn't compile until it has one. Enums (`display: block | flex | …`, `order`, `sticky` …) are a kind too, with their members as tokens. What else a common param says about itself (whether it takes a breakpoint, whether it needs a box) becomes an exhaustive `match` too, so a new one asks for each answer.
3. **A document's own components get the same.** Their params are declared with the same type names and enums (spec §7.3), so a `compmd` definition's `size: small | large` or `gap: space` param, and a host-language component's, has its kind from its declaration, and its F009 says what it takes as the built-ins' do.
4. **`areas` becomes a type** (spec §7.3), so `box` declares `grid-template-areas: areas?` and the check by the param's name goes.
5. **F009 for a value is written in one place.** `refused(key, kind, value)` builds the message; no other code writes "takes" for a value. The other F009s, which are about how a value is written rather than which value it is (`display=contents` outside a fence, an expression for a `name` param), keep their own messages.
6. **The message form is F008's, with the hint last:** "`border` takes `none`, `subtle`, `strong` or a CSS border such as `1px solid #d0d7de`, so the param is treated as absent; did you mean `subtle`?" The wording for each kind is in [the table below](#the-kinds-and-their-wording).
7. **Tests tie the definition to everything that depends on it** (see [Guard rails](#guard-rails)): each kind's tokens and form example are accepted, its description names them, the spec's tables and `llms.txt` list the same tokens and params, and the renderer has a CSS value for every token.
8. **The processor writes what each param takes to `conformance/params.json`**, and a test fails when the file is out of date. It is how the renderer, written in TypeScript, checks itself against the processor, and what an editor's help at the caret can list later.
9. **`cargo make ci` runs the TypeScript packages' tests too** (`pnpm -r test`), so the one command checks the renderer against `params.json`.
10. **`Kind` is data an editor can use**, not only a message: its tokens, forms and form example are public, so completions and hovers come from the same definition (see [Editors](#editors-lsp-and-tree-sitter)).
11. **A diagnostic with a suggestion carries it as a fix**, `fix: {range, text}`, besides its message, in this change: an optional field of diagnostics (spec §10.5) and their JSON, which a language server turns into a quick fix and the web editor into a click. F002, F006, F008 and F009, which suggest a name or a token, give one.

## Where it can drift today

What a param takes is written down in eight places, by hand:

| Where | What it holds | What keeps it in step |
|---|---|---|
| `validate/grammar.rs` | each grammar's tokens, inline in `accepts` (the space tokens twice, in `is_space` and again for `Spaces`) | nothing |
| `validate/params.rs` | which type name is which grammar, in a `match` ending in `_ => None`: a new layout type name would refuse every value, silently | nothing |
| `validate/params.rs` | `grid-template-areas`, declared `string`, checked against the areas grammar by its name | nothing |
| `validate/universal.rs` | each common param's grammar | an exhaustive `match` |
| `core.comp.md` | `box`'s own params and their types and enums | it is read as a definition |
| spec §9.2 and §9.4 | the params, their values and the tokens, for writers | nothing |
| `llms.txt` | the same, for language models writing documents | nothing |
| `render/src/styles.ts` | the params it styles (`LAYOUT`, `COMMON`), and each token's CSS (`SPACE`, `SURFACE`, `BORDER`, `RADIUS`, `ORDER`) | nothing |

Messages that list tokens would be a seventh. A token added to the grammar but not to `styles.ts` renders as its own name, `border: hairline`, which CSS drops without a word.

## The design

### A kind of value

```rust
/// What a param's value may be: tokens, the CSS forms it also takes, and an example of those forms.
pub(crate) struct Kind {
    /// The words it takes, in the order a message lists them.
    pub tokens: &'static [&'static str],
    /// The forms it takes besides its tokens.
    pub forms: &'static [Form],
    /// A value of its forms, since a message lists every token but can't list a form's values: what "a CSS
    /// border" looks like. A kind of only tokens has none.
    pub form_example: Option<&'static str>,
}

/// A form of value beyond tokens, with how it is checked and how a message names it.
pub(crate) enum Form { Length, Size, Spaces, Color, Border, Tracks, Areas, AreaName, Ratio, Language, Number, Date, Json(Json), Name, Text }
```

- `Kind::accepts(value)`: one of `tokens`, or any of `forms`. The checks of the forms are today's (`is_length`, `is_color`, `is_border`, …), moved, not rewritten.
- `Kind::describe()`: every token in backticks, then each form's name, then "such as" and the form example, which shows what the form looks like: "`none`, `s`, `m`, `l` or a length such as `1rem`". The tokens are listed, so the example is never a token; a kind of only tokens (`order`, an enum) has no example and its message lists them all.
- `Kind::suggest(value)`: the nearest token, by the closeness rule names already use (`nearest`); for a kind with a `Color` form, CSS's color names count too.

The kinds are constants, one per row of the table below: `SPACE`, `SPACES`, `RADIUS`, `SURFACE`, `BORDER`, `COLOR`, `LENGTH`, `SIZE`, `TRACKS`, `AREAS`, `AREA_NAME`, `RATIO`, `LANGUAGE`, and for the plain types `NUMBER`, `BOOLEAN`, `DATE`, `ARRAY`, `OBJECT`, `NAME`, `STRING`. A keyword param or an enum is a kind made from its words: `Kind::one_of(members)`. A flag (`scroll`, `fill`) is `FLAG`, whose tokens are `true` and `false`, and which a bare flag also fits.

`Grammar` goes: each of its variants becomes a kind, and `Grammar::Keywords` becomes `one_of`.

### Every param has a kind

```rust
impl TypeName {
    pub(crate) fn kind(self) -> Kind { match self { Self::Space => SPACE, Self::Border => BORDER, /* every variant, no `_` */ } }
}
impl Universal {
    pub(crate) fn kind(self) -> Kind { /* every variant, as `takes` is today */ }
}
```

`TypeName::Enum` (only in `input<enum>`) and `TypeName::String` take any text, so their kind has no tokens and the `Text` form.

`grid-template-areas` gets the new type `areas`, so `box` declares `grid-template-areas: areas?` and the check by name goes. The spec adds `areas` to its type names in §7.3 and to §9.4.

**A document's own components.** The kind comes from the declaration, so it is found at run time for any component: `kind()` of a declared type is `TypeName::kind` for a type name, `Kind::one_of(members)` for an enum, and any value for a name the spec doesn't define (`DeclaredType::Other`), as now. A `compmd` definition

````md
```compmd component=badge
params:
  tone: info | warning | danger = info
  gap: space = s
---
…
```
````

called `:: badge[New] {tone=warn gap=wide}` gives "`tone` takes one of `info`, `warning`, `danger`, so the param is treated as absent; did you mean `warning`?" and "`gap` takes `none`, `s`, `m`, `l` or a length such as `1rem`, so the param is treated as absent". `params.json` lists only the fixed vocabulary, the common params and `box`'s, since a document's own components belong to the document; an editor asks the processor about those (see [Editors](#editors-lsp-and-tree-sitter)).

### One place writes the message

```rust
/// F009 for a value `key` doesn't take: what it takes, and the nearest token when one is close.
fn refused(&mut self, at: Location, key: &str, kind: &Kind, value: Option<&str>) -> Verdict
```

Both paths that refuse a value today, a common param's (`universal`) and a declared one's (`declared`), call it with the param's kind. `DeclaredType::written` stays for F030, which names an input's type rather than listing its values.

### Guard rails

What each test catches, by the change someone might make:

| A change | What fails until the rest follows |
|---|---|
| A new type name or common param | Compiling: the `match` in `kind()` has no arm for it, nor those of `takes_breakpoint` and `needs_box` |
| A new token in a kind | Nothing in the processor, since the check, the message and the suggestion read the same list. The spec test, until §9.2 or §9.4 and `llms.txt` list it; the renderer test, until `styles.ts` maps it to CSS |
| A token removed | The same two tests |
| A kind whose form example it doesn't take, or is a token, or a description missing a token | `every_kind_takes_its_example_and_names_its_tokens`, over every kind |
| A token a typo of which suggests another token | `each_token_is_suggested_for_a_typo_of_it`: one changed letter of each token suggests it |
| A new param of `box` in `core.comp.md` | The spec test, until §9 lists it; the renderer test, until `LAYOUT` has it |
| `params.json` not written again after any of these | `params_json_is_up_to_date`, which says how to write it |

- **The spec test** (`tests/spec_values.rs`) reads the tables of spec §9.2 and §9.4 from `compmd-spec-v1.md`: every common param and `box` param is in a row, and the tokens a row lists in backticks are the kind's tokens. It reads `llms.txt` the same way: every param is named, and the line naming a param with tokens lists them. The spec stays the reference for writers, and the test makes sure it says what the processor does.
- **`conformance/params.json`** lists every common param and every `box` param with its kind: `{"border": {"tokens": ["none", "subtle", "strong"], "forms": ["border"], "form_example": "1px solid #d0d7de", "breakpoints": true}, …}`. A Rust test compares it with what the processor gives and fails with the command that writes it again (`COMPMD_WRITE=1 cargo test`, a variable the test reads).
- **The renderer test** (`render/src/styles.test.ts`) reads `params.json`: `LAYOUT` and `COMMON` are exactly its params, and every token of every param either has its own CSS in `styles.ts` or is a CSS keyword of that property (`order=first` maps, `align-self=center` passes through). The few that pass through are listed in the test, so a new token is a decision, not an accident.

### The kinds and their wording

As agreed: "`<key>` takes …".

| Kind | Params | Takes |
|---|---|---|
| space | `padding-*-start`, `padding-*-end`, `gap` | `none`, `s`, `m`, `l` or a length such as `1rem` |
| spaces | `padding`, `padding-inline`, `padding-block` | one or two of `none`, `s`, `m`, `l` or a length, such as `s l` |
| radius | `border-radius` | `none`, `s`, `m`, `l`, `full` or a length such as `6px` |
| surface | `background` | `none`, `subtle`, `strong`, `accent` or a CSS color such as `#f6f8fa` |
| border | `border` | `none`, `subtle`, `strong` or a CSS border such as `1px solid #d0d7de` |
| color | type `color` | a CSS color such as `#0b7285` or `teal` |
| length | type `length` | a length in `px`, `rem`, `em` or `%`, such as `1.5rem`, or `0` |
| size | `width`, `height`, `min-width`, `max-width`, the insets | `auto` or a length such as `20rem` |
| tracks | `grid-template-columns`, `grid-template-rows` | grid tracks such as `1fr 2fr` or `repeat(auto-fill, minmax(12rem, 1fr))` |
| areas | `grid-template-areas` | quoted rows of area names, all as long, such as `'head head' 'side main'` |
| area name | `grid-area` | an area name such as `side` |
| ratio | `aspect-ratio` | `none` or a width over a height such as `16/9` |
| language | `lang` | a language tag such as `en` or `de-CH` |
| one of | `order`, `align-self`, `justify-self`, `sticky`, `show`, `hide`, `dir`, enums | one of `first`, `last`, `none` |
| flag | `scroll`, `fill` | nothing: it is written alone, or as `true` or `false` |
| number, boolean, date | the types | a number such as `12` or `0.5`; `true` or `false`; a date, `YYYY-MM-DD`, such as `2026-10-29` |
| array, object | the types | an array (an object) as JSON, such as `[1, 2]`, or an expression |
| name | type `name` | a name, written as is, such as `row` |

## Adding a param or a value

Each change below is complete: every file it touches, and what fails if one is left out.

### A token: `border=hairline`

A border fainter than `subtle`, a theme's to change like the others.

1. **`processor/src/validate/values.rs`**, the only change in the processor: the check, the message and the suggestion follow.

   ```rust
   pub(crate) const BORDER: Kind = Kind {
       tokens: &["none", "subtle", "strong", "hairline"],
       forms: &[Form::Border],
       form_example: Some("1px solid #d0d7de"),
   };
   ```

   The new token goes in `tokens`; `form_example` shows the other way to write a border, a CSS border, and doesn't change. The message is then "`border` takes `none`, `subtle`, `strong`, `hairline` or a CSS border such as `1px solid #d0d7de`", and `border=hairlin` gets "did you mean `hairline`?".

2. **`compmd-spec-v1.md`**: the token in §9.2's row for `border`, and a row in §9.4's table.

   ```md
   | `border`  | a token (`none`, `subtle`, `strong`, `hairline`) or a CSS border | `border=subtle`, `border="1px solid"` |
   | `border=hairline` | `--compmd-border-hairline` | `1px solid color-mix(in srgb, currentColor 10%, transparent)` |
   ```

3. **`llms.txt`**: ``- look: `border` (`none subtle strong hairline` or CSS), …``.
4. **`conformance/params.json`**: written again, `COMPMD_WRITE=1 cargo test params_json`.
5. **`render/src/styles.ts`**: its CSS.

   ```ts
   const BORDER: Record<string, string> = {
     …
     hairline: "var(--compmd-border-hairline, 1px solid color-mix(in srgb, currentColor 10%, transparent))",
   };
   ```

Left out: the spec or `llms.txt`, and the spec test fails naming the token and the file; `params.json`, and its test fails with the command; `styles.ts`, and the renderer test fails: "`border=hairline` has no CSS".

### A common param: `text-align`

`start`, `center` or `end`, with breakpoints, applied to the box.

1. **`processor/src/validate/universal.rs`**: the param, then the answers the compiler asks for.

   ```rust
   named! {
       pub(crate) enum Universal {
           …
           TextAlign = "text-align",
       }
   }

   // in `kind`
   Self::TextAlign => Kind::one_of(&["start", "center", "end"]),
   // in `takes_breakpoint`
   Self::TextAlign => true,
   // in `needs_box`
   Self::TextAlign => true,
   ```

   Without the three arms it doesn't compile. Nothing else in the processor changes: a common param is found by its name, checked, and copied to the expanded tree as the others are.
2. **`compmd-spec-v1.md`**, a row in §9.2:

   ```md
   | `text-align` | `start`, `center`, `end` | `text-align=center` |
   ```

3. **`llms.txt`**: the param in the list of common params it belongs with.
4. **`conformance/params.json`**: written again.
5. **`render/src/styles.ts`**: the param in `COMMON`. Its tokens are CSS's own words for `text-align`, so they pass through, and the renderer test lists them as passing through:

   ```ts
   // render/src/styles.test.ts
   const PASSED_THROUGH = { …, "text-align": ["start", "center", "end"] };
   ```

Left out: as for a token, each of the tests names what is missing, and the renderer test also fails while `COMMON` and `params.json` differ.

### A param of `box`: `align-content`

1. **`processor/src/core.comp.md`**: `align-content: start | center | end | between = start` in `box`'s signature, and in `grid`'s if it passes it on. The kind comes from the enum.
2. **`compmd-spec-v1.md`**: the param in §9's list of `box`'s params.
3. **`llms.txt`**, **`conformance/params.json`**: as above.
4. **`render/src/styles.ts`**: the param in `LAYOUT`, and `between` mapped to `space-between` (`CONTENT_ALIGNMENT`, as `justify-content` does); the others pass through.

### A kind or a type name: `fraction`

A type for a number from 0 to 1, say for an `opacity` param.

1. **`processor/src/tree.rs`**: `Fraction = "fraction"` in `TypeName`; the compiler then asks for its arm in `TypeName::kind`.
2. **`processor/src/validate/values.rs`**: the kind, and its form if it is new, with the form's check and its name in messages.

   ```rust
   pub(crate) const FRACTION: Kind = Kind { tokens: &[], forms: &[Form::Fraction], form_example: Some("0.5") };
   ```

   `every_kind_takes_its_example_and_names_its_tokens` covers it by being in `Kind::ALL`, which the kinds are listed in once.
3. **`compmd-spec-v1.md`**: the type name in §7.3, and its grammar in §9.4.
4. Then a param that uses it, as above.

## Editors: LSP and tree-sitter

An editor's support for CompMD has three parts, and this work is the base of one of them.

| Part | What it does | Built from | Relation to this work |
|---|---|---|---|
| **A tree-sitter grammar** | highlighting, folding, selecting a block, in Neovim, Helix, Zed and Emacs, as you type | a grammar of CompMD's syntax (component lines, params, fences, slots, interpolations), written for tree-sitter in JavaScript | None in code: tree-sitter parses syntax and can't check a value. The conformance examples can be its test corpus, so it parses what the processor parses |
| **A language server (LSP)** | diagnostics, completions, hovers and quick fixes, in VS Code, Neovim, Emacs and any editor with an LSP client | a small Rust program over the processor crate, keeping each open file as the editor binding does | This work is the core of its completions, hovers and quick fixes: a param's values are `Kind`'s tokens and forms, its hover is `describe()`, and a quick fix is the suggestion |
| **The web editor** | the same, in the browser, through `@compmd/processor` | the binding | The same, through the binding |

**What a language server needs, and what changes here because of it:**

- **Structured data, not messages to parse.** `Kind` is public (`compmd::Kind`, with `tokens`, `forms`, `form_example`), so a server builds completion items from it rather than from a message. The message is for people.
- **A fix with each suggestion.** A diagnostic with a suggestion carries it as `fix: {range, text}` besides its message: `subtl` at 3:22–3:27 → `subtle`. A server turns it into a code action, and the web editor into a click. This adds an optional field to diagnostics (spec §10.5), in this change (decision 11).
- **What is at the caret** (the component, the param, its kind, its default and its `doc`) is the query the caret help adds next; completions and hovers in a server are that query, so the server is a thin layer over it.
- **Positions.** LSP counts columns in UTF-16 by default, as the binding does; the binding's line index (`wasm/src/text.rs`) moves into the processor crate when a server is built, so both share it. Not in this change.

So tree-sitter is work beside this, sharing only the conformance examples. A language server is work on top of it: with `Kind`, the fix and the caret query, it is diagnostics and completions over what the processor already gives, plus the protocol.

## Not in this change

- **Help at the caret.** An editor that lists a param's values as you type asks the processor what is at the caret; that is the next work, on top of `Kind`.
- **A language server and a tree-sitter grammar** (see above).
- **CI on GitHub**, which is its own work; `cargo make ci` runs both languages' checks here, so CI can run that one command.
- **Codes, ranges and recovery.** An F009 is still at the value, and the param is still treated as absent.
- **F024**, a typed expression of the wrong type, names CEL types (`string`, `double`) and stays as it is.

## Plan

1. `validate/values.rs`: `Kind`, `Form`, the kinds and `Kind::ALL`, `accepts`, `describe` and `suggest`, with `grammar.rs`'s checks moved into it; `TypeName::kind`, and `Universal::kind`, `takes_breakpoint` and `needs_box` exhaustive; `refused` on both paths. Tests of each kind's message and suggestion, of a document's own component's params, and the guard-rail tests over every kind.
2. The type `areas`: the spec (§7.3, §9.4), `core.comp.md`, and the check by name removed.
3. `fix` on diagnostics: the spec (§10.5), the diagnostic and its JSON, the binding's types, F002, F006, F008 and F009 giving one, and the editor showing it.
4. `conformance/params.json` and its test; the spec and `llms.txt` test.
5. The renderer test against `params.json`, and whatever it finds missing in `styles.ts`.
6. `pnpm -r test` in `cargo make ci`, and it green.
