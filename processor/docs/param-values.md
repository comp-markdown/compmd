# What a param takes

**Status: built.**

A param value outside its type is F009. Its message used to say only "`padding` doesn't take this value" for most params; it now says what the param takes. What a param takes is a single definition, from which the check, the message, the suggestion and the renderer's tests all come, so adding a token, a type or a param can't leave one of them behind.

## Decisions

1. **One definition per kind of value.** Each kind (`space`, `border`, `color`, `tracks`, …) is one constant of [`Kind`](../src/values.rs) in `src/values.rs` (public as `compmd::values`): its tokens, the forms it also takes, and examples of those forms. Checking a value, describing the kind in a message and suggesting a token are three functions of that constant. No token list is written anywhere else in the processor.
2. **Every param gets its kind by an exhaustive `match`.** A declared type name (`TypeName`) and a common param (`Universal`) each map to a kind in a `match` with no `_` arm, so a new type name or common param doesn't compile until it has one. Enums (`display: block | flex | …`, `order`, `sticky` …) are a kind too, with their members as tokens. What else a common param says about itself (whether it takes a breakpoint, needs a box, is a common param or an attribute) is an exhaustive `match` too, so a new one asks for each answer.
3. **A document's own components get the same.** Their params are declared with the same type names and enums (spec §7.3), so a `compmd` definition's `size: small | large` or `gap: space` param, and a host-language component's, has its kind from its declaration, and its F009 says what it takes as the built-ins' do.
4. **`areas` is a type** (spec §7.3), so `box` declares `grid-template-areas: areas?`, and the check by the param's name is gone.
5. **F009 for a value is written in one place**, `Checker::refused` in `validate/params.rs`; no other code writes "takes" for a value. The other F009s, which are about how a value is written rather than which value it is (`display=contents` outside a fence, an expression for a `name` param), keep their own messages.
6. **The message form is F008's, with the hint last:** "`border` takes `none`, `subtle`, `strong` or a CSS border, such as `1px solid #d0d7de`, so the param is treated as absent; did you mean `subtle`?" The wording for each kind is in [the table below](#the-kinds-and-their-wording).
7. **Tests tie the definition to everything that depends on it** (see [Guard rails](#guard-rails)): each kind's tokens and examples are accepted, its description names them, the spec and `llms.txt` list the same params and tokens, and the renderer has CSS for every token.
8. **The fixed vocabulary is a public function**, `compmd::values::vocabulary()`: the common params, the attributes and `box`'s params, each with what it takes. `conformance/params.json` is its JSON, and a test fails when the file is out of date. It is how the renderer, written in TypeScript, checks itself against the processor, and what an editor's help at the caret can list.
9. **`cargo make ci` builds and tests the TypeScript packages too** (its `packages` task), so the one command checks the renderer against `params.json`.
10. **`Kind` is data an editor can use**, not only a message: its fields are public, so completions and hovers come from the same definition (see [Editors](#editors-lsp-and-tree-sitter)).
11. **A diagnostic with a suggestion carries it as a fix**, `fix: {range, text}`, besides its message: an optional field of diagnostics (spec §10.5) and their JSON, which a language server turns into a quick fix and the web editor into a click. F002, F006, F008 and F009 give one when they suggest a name or a token.

## Where it drifted, and what keeps it in step now

What a param takes was written down in ten places, by hand:

| Where | What it holds | What keeps it in step now |
|---|---|---|
| `validate/grammar.rs` (gone) | each grammar's tokens, inline (the space tokens twice) | the kinds, in `src/values.rs`, and the forms' checks, moved unchanged to `src/values/forms.rs` |
| `validate/params.rs` | which type name is which grammar, in a `match` ending in `_ => None` | `TypeName::kind`, exhaustive |
| `validate/params.rs` | `grid-template-areas`, declared `string`, checked against the areas grammar by its name | the type `areas` |
| `validate/universal.rs` | each common param's grammar | `Universal::kind`, exhaustive, as it was |
| `core.comp.md` | `box`'s own params, their types and enums | read as a definition, and into `vocabulary()` |
| spec §7.3 and §8.2 | the type names | a test over `TypeName::NAMES` (§7.3's grammar) |
| spec §9 | a copy of `box`'s signature | a test: it is `core.comp.md`'s |
| spec §9.2 | the common params and their tokens | a test: each is in a row that lists its tokens |
| `llms.txt` | the params and tokens, for language models writing documents | a test: each is named on a line that lists its tokens |
| `render/src/styles.ts` | the params it styles (`LAYOUT`, `COMMON`, `ATTRIBUTES`), and each token's CSS | a test against `params.json` |

A token added to the processor but not to `styles.ts` would render as its own name, `border: hairline`, which CSS drops without a word; the renderer test now fails instead.

## The design

### A kind of value

```rust
/// A kind of value: the words it takes, the forms it also takes, and examples of those forms.
pub struct Kind<'a> {
    /// The words it takes, in the order a message lists them.
    pub tokens: Cow<'a, [&'a str]>,
    /// The forms it takes besides its tokens.
    pub forms: &'static [Form],
    /// Values of its forms, since a message lists every token but can't list a form's values: what "a CSS border"
    /// looks like. A kind of only tokens has none.
    pub examples: &'static [&'static str],
    /// How many space-separated values it takes, each a token or of a form: 2 for `padding`'s `s l`.
    pub up_to: usize,
    /// Whether a flag, written without a value, is one: `true`.
    pub flag: bool,
    /// Whether a quoted string may hold it; a number, a boolean and a name are written bare (spec §5.1).
    pub quoted: bool,
}

/// A form of value beyond tokens, with how it is checked and how a message names it.
pub enum Form { Length, Color, Border, Tracks, Areas, AreaName, Ratio, Language, Number, Date, Array, Object, Name, Text, Any }
```

- `Kind::accepts(value)`: a value as written (bare, quoted, JSON, a flag) is one of `tokens`, or of one of `forms`, as many times as `up_to` allows. A string with interpolations is checked when it is rendered, as before. The forms' checks are the old grammars', moved, not rewritten.
- `Kind::describe()`: every token in backticks, then each form's phrase, then ", such as" and the examples: "`none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `1rem`". The tokens are all listed, so an example is never a token; a kind of only tokens (`order`, an enum) has no example.
- `Kind::suggest(written)`: the nearest token, by the closeness rule names already use (`nearest`); for a kind with a color or border form, CSS's color names count too.

The kinds are constants, one per row of [the table below](#the-kinds-and-their-wording), listed once in `Kind::ALL`: `SPACE`, `SPACES`, `RADIUS`, `SURFACE`, `BORDER`, `COLOR`, `LENGTH`, `SIZE`, `TRACKS`, `AREAS`, `AREA_NAME`, `RATIO`, `LANGUAGE`, and for the plain types `NUMBER`, `BOOLEAN`, `DATE`, `ARRAY`, `OBJECT`, `NAME`, `TEXT` and `ANY`. A keyword param or an enum is a kind made from its words, `Kind::one_of(members)`. `scroll` and `fill` are `BOOLEAN`, whose tokens are `true` and `false` and which a flag also is.

### Every param has a kind

```rust
impl TypeName {
    pub fn kind(self) -> Kind<'static> { match self { Self::Space => Kind::SPACE, Self::Border => Kind::BORDER, /* every variant, no `_` */ } }
}
impl Universal {
    pub(crate) fn kind(self) -> Kind<'static> { /* every variant */ }
}
impl Kind<'_> {
    pub fn declared(declared: &DeclaredType) -> Kind<'_> { /* a type name's kind, an enum's members, or any value */ }
}
```

`TypeName::String` takes any text; `TypeName::Enum` (only in `input<enum>`) and a name the spec doesn't define (`DeclaredType::Other`) take any value.

**A document's own components.** The kind comes from the declaration, so it is found for any component. A `compmd` definition

````md
```compmd component=badge
params:
  tone: info | warning | danger = info
  gap: space = s
---
…
```
````

called `:: badge[New] {tone=warnign gap=wide}` gives "`tone` takes `info`, `warning` or `danger`, so the param is treated as absent; did you mean `warning`?" and "`gap` takes `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `1rem`, so the param is treated as absent". (`tone=warn` gets no suggestion: three of `warning`'s seven letters are missing, more than the closeness rule allows.) `params.json` lists only the fixed vocabulary, since a document's own components belong to the document; an editor asks the processor about those.

### One place writes the message

```rust
/// F009 for a value `key` doesn't take: what it takes, and the nearest token when one is close, with the fix that
/// writes it. The only place that says what a param takes.
fn refused(&mut self, param: &Param, key: &str, kind: &Kind) -> Verdict
```

Both paths that refuse a value, a common param's and a declared one's, call it with the param's kind. Its fix replaces the value as written, quotes and all, by the token. `DeclaredType::written` stays for F030, which names an input's type rather than listing its values.

### Guard rails

What each test catches, by the change someone might make:

| A change | What fails until the rest follows |
|---|---|
| A new type name | Compiling: `TypeName::kind` and `Type::declared` (the type an expression gets) have no arm for it. Then `the_specs_type_grammar_has_every_type_name`, until spec §7.3's grammar has it |
| A new common param | Compiling: `Universal::kind`, `takes_breakpoint`, `needs_box` and `is_common` have no arm for it. Then the spec and `llms.txt` tests, and the renderer's, until `COMMON` has it |
| A new token in a kind | Nothing in the processor, since the check, the message and the suggestion read the same list. The spec and `llms.txt` tests, until they list it; the renderer test, until `styles.ts` gives it CSS |
| A token removed | The same tests |
| A kind whose example it doesn't take, or is a token, or whose description misses a token | `every_kind_takes_its_examples_and_names_its_tokens`, over `Kind::ALL` |
| A token a typo of which suggests another token | `each_token_is_suggested_for_a_typo_of_it`: each token with its first letter left out suggests it |
| A new param of `box` in `core.comp.md` | `the_specs_copy_of_box_is_core_s`, until the spec's copy has it; the `llms.txt` test; the renderer test, until `LAYOUT` has it |
| `params.json` not written again after any of these | `params_json_is_up_to_date`, which says how to write it |

- **The kinds' tests** are in `src/values.rs` and `src/values/forms.rs`; the messages and fixes, end to end, in `tests/messages.rs`.
- **The vocabulary's tests** are `tests/vocabulary.rs`. They read spec §9.2's table (a common param's row lists every token in backticks), §7.3's type grammar, and the spec's copy of `box`'s signature, and `llms.txt` (a line naming a param in backticks lists every token in backticks). The spec stays the reference for writers, and the tests make sure it says what the processor does.
- **`conformance/params.json`** is `vocabulary()` as JSON: `{"common": [{"name": "border", "tokens": ["none", "subtle", "strong"], "forms": ["border"], "examples": ["1px solid #d0d7de"], "upTo": 1, "flag": false, "quoted": true, "breakpoints": true}, …], "attributes": […], "layout": […]}`. Written again with `COMPMD_WRITE=1 cargo test --test vocabulary`.
- **The renderer test** (`render/src/vocabulary.test.ts`) reads `params.json`: `COMMON`, `LAYOUT` and `ATTRIBUTES` are exactly its params, and every token of a common or layout param gets CSS. A token whose CSS is the token itself, a CSS keyword of the property (`align-self=center`), is listed in the test, and so is one that means no CSS (`sticky=none`), so a new token is a decision, not an accident.

### The kinds and their wording

"`<key>` takes …":

| Kind | Params | Takes |
|---|---|---|
| space | `padding-*-start`, `padding-*-end`, `gap` | `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `1rem` |
| spaces | `padding`, `padding-inline`, `padding-block` | one or two of `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `s l` |
| radius | `border-radius` | `none`, `s`, `m`, `l`, `full` or a length in …, such as `6px` |
| surface | `background` | `none`, `subtle`, `strong`, `accent` or a CSS color, such as `#f6f8fa` |
| border | `border` | `none`, `subtle`, `strong` or a CSS border, such as `1px solid #d0d7de` |
| color | type `color` | a CSS color, such as `#0b7285` or `teal` |
| length | type `length` | a length in `px`, `rem`, `em` or `%`, such as `1.5rem` |
| size | `width`, `height`, `min-width`, `max-width`, the insets | `auto` or a length in …, such as `20rem` |
| tracks | `grid-template-columns`, `grid-template-rows` | grid tracks, such as `1fr 2fr` or `repeat(auto-fill, minmax(12rem, 1fr))` |
| areas | `grid-template-areas` | quoted rows of area names, all as long, such as `'head head' 'side main'` |
| area name | `grid-area` | an area name, such as `side` |
| ratio | `aspect-ratio` | `none` or a width over a height, such as `16/9` |
| language | `lang` | a language tag, such as `en` or `de-CH` |
| one of | `order`, `align-self`, `justify-self`, `sticky`, `show`, `hide`, `dir`, enums | `first`, `last` or `none` |
| boolean | `scroll`, `fill`, type `boolean` | `true` or `false`, or nothing, as a flag |
| number, date | the types | a number, such as `12` or `0.5`; a date (`YYYY-MM-DD`), such as `2026-10-29` |
| array, object | the types | an array, as JSON, such as `[1, 2]`; an object, as JSON, such as `{"a": 1}` |
| name | type `name` | a name, written as is, such as `row` |

## Adding a param or a value

Each change below is complete: every file it touches, and what fails if one is left out.

### A token: `border=hairline`

A border fainter than `subtle`, a theme's to change like the others.

1. **`processor/src/values.rs`**, the only change in the processor: the check, the message and the suggestion follow.

   ```rust
   pub const BORDER: Self = kind(
       &["none", "subtle", "strong", "hairline"],
       &[Form::Border],
       &["1px solid #d0d7de"],
   );
   ```

   The new token goes in the tokens; the example shows the other way to write a border, a CSS border, and doesn't change. The message is then "`border` takes `none`, `subtle`, `strong`, `hairline` or a CSS border, such as `1px solid #d0d7de`", and `border=hairlin` gets "did you mean `hairline`?".

2. **`compmd-spec-v1.md`**: the token in §9.2's row for `border`, and a row in §9.4's table.

   ```md
   | `border`  | a token (`none`, `subtle`, `strong`, `hairline`) or a CSS border | `border=subtle`, `border="1px solid"` |
   | `border=hairline` | `--compmd-border-hairline` | `1px solid color-mix(in srgb, currentColor 10%, transparent)` |
   ```

3. **`llms.txt`**: ``- look: `border` (`none subtle strong hairline` or CSS), …``.
4. **`conformance/params.json`**: written again, `COMPMD_WRITE=1 cargo test --test vocabulary`.
5. **`render/src/styles.ts`**: its CSS.

   ```ts
   const BORDER: Record<string, string> = {
     …
     hairline: "var(--compmd-border-hairline, 1px solid color-mix(in srgb, currentColor 10%, transparent))",
   };
   ```

Left out: the spec or `llms.txt`, and its test fails naming the param and the tokens; `params.json`, and its test fails with the command; `styles.ts`, and the renderer test fails: "`border=hairline` passes through as it is; list it if that is right".

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
   Self::TextAlign => Kind::one_of(&["start", "center", "end"][..]),
   // in `takes_breakpoint`, `needs_box` and `is_common`
   Self::TextAlign => true,
   ```

   Without the four arms it doesn't compile. Nothing else in the processor changes: a common param is found by its name, checked, and copied to the expanded tree as the others are.
2. **`compmd-spec-v1.md`**, a row in §9.2:

   ```md
   | `text-align` | `start`, `center`, `end` | `text-align=center` |
   ```

3. **`llms.txt`**: the param, with its tokens, in the list of common params it belongs with: ```text-align` (`start center end`)``.
4. **`conformance/params.json`**: written again.
5. **`render/src/styles.ts`**: the param in `COMMON`. Its tokens are CSS's own words for `text-align`, so they pass through, and the renderer test lists them as passing through:

   ```ts
   // render/src/vocabulary.test.ts
   const PASSED_THROUGH = { …, "text-align": ["start", "center", "end"] };
   ```

Left out: as for a token, each test names what is missing, and the renderer test also fails while `COMMON` and `params.json` differ.

### A param of `box`: `align-content`

1. **`processor/src/core.comp.md`**: `align-content: start | center | end | between = start` in `box`'s signature, and in `grid`'s if it passes it on. Its kind comes from the enum.
2. **`compmd-spec-v1.md`**: the same line in the spec's copy of `box`'s signature (§9).
3. **`llms.txt`**: ``…, `align-content=start|center|end|between``` with `box`'s other params.
4. **`conformance/params.json`**: written again.
5. **`render/src/styles.ts`**: the param in `LAYOUT`, and `between` mapped to `space-between` (`CONTENT_ALIGNMENT`, as for `justify-content`); the others pass through, listed in the renderer test.

### A type name and a kind: `fraction`

A type for a number from 0 to 1, say for an `opacity` param.

1. **`processor/src/tree.rs`**: `Fraction = "fraction"` in `TypeName`. The compiler then asks for its arm in `TypeName::kind` (`src/values.rs`) and in `Type::declared` (`src/cel/types.rs`, the type an expression gets: here `double`).
2. **`processor/src/values.rs`** and **`src/values/forms.rs`**: the kind, and its form if it is new, with the form's check and its phrase.

   ```rust
   pub const FRACTION: Self = kind(&[], &[Form::Fraction], &["0.5"]);
   ```

   Listed in `Kind::ALL`, so the guard-rail tests cover it.
3. **`compmd-spec-v1.md`**: the type name in §7.3's grammar (its test fails until it is there), its expression type in §8.2, and its grammar in §9.4.
4. Then a param that uses it, as above.

## Editors: LSP and tree-sitter

An editor's support for CompMD has three parts, and this work is the base of one of them.

| Part | What it does | Built from | Relation to this work |
|---|---|---|---|
| **A tree-sitter grammar** | highlighting, folding, selecting a block, in Neovim, Helix, Zed and Emacs, as you type | a grammar of CompMD's syntax (component lines, params, fences, slots, interpolations), written for tree-sitter in JavaScript | None in code: tree-sitter parses syntax and can't check a value. The conformance examples can be its test corpus, so it parses what the processor parses |
| **A language server (LSP)** | diagnostics, completions, hovers and quick fixes, in VS Code, Neovim, Emacs and any editor with an LSP client | a small Rust program over the processor crate, keeping each open file as the editor binding does | This work is the core of its completions, hovers and quick fixes: a param's values are its `Kind`'s tokens and forms, its hover is `describe()`, and a quick fix is the diagnostic's `fix` |
| **The web editor** | the same, in the browser, through `@compmd/processor` | the binding | The same, through the binding; its diagnostics list offers each fix as a button |

**What a language server needs, and what changed here because of it:**

- **Structured data, not messages to parse.** `compmd::values::Kind`'s fields and `vocabulary()` are public, so a server builds completion items from them rather than from a message. The message is for people.
- **A fix with each suggestion.** A diagnostic with a suggestion carries it as `fix: {range, text}`: `subtl` at 3:17–3:22 → `subtle`. A server turns it into a code action.
- **What is at the caret** (the component, the param, its kind, its default and its `doc`) is the query the caret help adds next; completions and hovers in a server are that query, so the server is a thin layer over it.
- **Positions.** LSP counts columns in UTF-16 by default, as the binding does; the binding's line index (`wasm/src/text.rs`) moves into the processor crate when a server is built, so both share it. Not in this change.

So tree-sitter is work beside this, sharing only the conformance examples. A language server is work on top of it: with `Kind`, the fix and the caret query, it is diagnostics and completions over what the processor already gives, plus the protocol.

## Not in this change

- **Help at the caret.** An editor that lists a param's values as you type asks the processor what is at the caret; that is the next work, on top of `Kind` and `vocabulary()`.
- **A language server and a tree-sitter grammar** (see above).
- **CI on GitHub**, which is its own work; `cargo make ci` runs both languages' checks here, so CI can run that one command.
- **Codes, ranges and recovery.** An F009 is still at the value, and the param is still treated as absent.
- **F024**, a typed expression of the wrong type, names CEL types (`string`, `double`) and stays as it is.
