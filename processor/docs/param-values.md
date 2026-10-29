# What a param takes

**Status: design, for review.**

A param value outside its type is F009, and today the message often says only "`padding` doesn't take this value". This design makes every F009 say what the param takes, and makes what a param takes a single definition, from which the check, the message, the suggestion and the renderer's tests all come. Adding a token, a type or a param then can't leave one of them behind.

## Decisions

1. **One definition per kind of value.** Each kind (`space`, `border`, `color`, `tracks`, …) is one entry in one table in `validate/values.rs`: its tokens, the CSS forms it also takes, and one example. Checking a value, describing the kind in a message and suggesting a token are three functions of that entry. No token list is written anywhere else in the processor.
2. **Every param gets its kind by an exhaustive `match`.** A declared type name (`TypeName`) and a common param (`Universal`) each map to a kind in a `match` with no `_` arm, so a new type name or common param doesn't compile until it has one. Enums (`display: block | flex | …`, `order`, `sticky` …) are a kind too, with their members as tokens.
3. **F009 for a value is written in one place.** `refused(key, kind, value)` builds the message; no other code writes "takes" for a value. The other F009s, which are about how a value is written rather than which value it is (`display=contents` outside a fence, an expression for a `name` param), keep their own messages.
4. **The message form is F008's, with the hint last:** "`border` takes `none`, `subtle`, `strong` or a CSS border such as `1px solid #d0d7de`, so the param is treated as absent; did you mean `subtle`?" The wording for each kind is in [the table below](#the-kinds-and-their-wording).
5. **Tests tie the definition to everything that depends on it** (see [Guard rails](#guard-rails)): each kind's example and tokens are accepted, its description names them, the spec's tables list the same tokens and params, and the renderer has a CSS value for every token.
6. **The processor writes what each param takes to `conformance/params.json`**, and a test fails when the file is out of date. It is how the renderer, written in TypeScript, checks itself against the processor, and what an editor's help at the caret can list later.

## Where it can drift today

What a param takes is written down in six places, by hand:

| Where | What it holds | What keeps it in step |
|---|---|---|
| `validate/grammar.rs` | each grammar's tokens, inline in `accepts` (the space tokens twice, in `is_space` and again for `Spaces`) | nothing |
| `validate/params.rs` | which type name is which grammar, in a `match` ending in `_ => None`: a new layout type name would refuse every value, silently | nothing |
| `validate/params.rs` | `grid-template-areas`, declared `string`, checked against the areas grammar by its name | nothing |
| `validate/universal.rs` | each common param's grammar | an exhaustive `match` |
| `core.comp.md` | `box`'s own params and their types and enums | it is read as a definition |
| spec §9.2 and §9.4 | the params, their values and the tokens, for writers | nothing |
| `render/src/styles.ts` | the params it styles (`LAYOUT`, `COMMON`), and each token's CSS (`SPACE`, `SURFACE`, `BORDER`, `RADIUS`, `ORDER`) | nothing |

Messages that list tokens would be a seventh. A token added to the grammar but not to `styles.ts` renders as its own name, `border: hairline`, which CSS drops without a word.

## The design

### A kind of value

```rust
/// What a param's value may be: tokens, the CSS forms it also takes, and an example of one it takes.
pub(crate) struct Kind {
    /// The words it takes, in the order a message lists them.
    pub tokens: &'static [&'static str],
    /// The forms it takes besides its tokens.
    pub forms: &'static [Form],
    /// A value it takes that isn't a token, for the message; or a token when it takes only tokens.
    pub example: &'static str,
}

/// A form of value beyond tokens, with how it is checked and how a message names it.
pub(crate) enum Form { Length, Size, Spaces, Color, Border, Tracks, Areas, AreaName, Ratio, Language, Number, Date, Json(Json), Name, Text }
```

- `Kind::accepts(value)`: one of `tokens`, or any of `forms`. The checks of the forms are today's (`is_length`, `is_color`, `is_border`, …), moved, not rewritten.
- `Kind::describe()`: the tokens in backticks, then each form's name, then "such as" and the example: "`none`, `s`, `m`, `l` or a length such as `1rem`".
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

`grid-template-areas` gets the type `areas` (spec §7.3), so `box` declares `grid-template-areas: areas?` and the check by name goes. This adds a type name to the spec; see [Open questions](#open-questions).

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
| A new type name or common param | Compiling: the `match` in `kind()` has no arm for it |
| A new token in a kind | Nothing in the processor, since the check, the message and the suggestion read the same list. The spec test, until §9.2 or §9.4 lists it; the renderer test, until `styles.ts` maps it to CSS |
| A token removed | The same two tests |
| A kind whose example it doesn't take, or a description missing a token | `every_kind_takes_its_example_and_names_its_tokens`, over every kind |
| A token a typo of which suggests another token | `each_token_is_suggested_for_a_typo_of_it`: one changed letter of each token suggests it |
| A new param of `box` in `core.comp.md` | The spec test, until §9 lists it; the renderer test, until `LAYOUT` has it |
| `params.json` not written again after any of these | `params_json_is_up_to_date`, which says how to write it |

- **The spec test** (`tests/spec_values.rs`) reads the tables of spec §9.2 and §9.4 from `compmd-spec-v1.md`: every common param and `box` param is in a row, and the tokens a row lists in backticks are the kind's tokens. The spec stays the reference for writers, and the test makes sure it says what the processor does.
- **`conformance/params.json`** lists every common param and every `box` param with its kind: `{"border": {"tokens": ["none", "subtle", "strong"], "forms": ["border"], "example": "1px solid #d0d7de", "breakpoints": true}, …}`. A Rust test compares it with what the processor gives and fails with the command that writes it again (`COMPMD_WRITE=1 cargo test`, a variable the test reads).
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

## Not in this change

- **Help at the caret.** An editor that lists a param's values as you type can read `params.json`, or a call of the binding giving the same; that is its own work.
- **Codes, ranges and recovery.** Only messages change: an F009 is still at the value, and the param is still treated as absent.
- **F024**, a typed expression of the wrong type, names CEL types (`string`, `double`) and stays as it is.

## Open questions

1. **A type `areas` in the spec** (§7.3), so `grid-template-areas` is checked by its type rather than by its name. The alternative is to keep the special case, inside `kind()` for that one param, with a test of it. I recommend the type: it is one line in the spec, and a definition's own param can then take areas too.
2. **The renderer test in `cargo make ci`.** CI runs only the Rust checks, so the renderer test runs with `pnpm test`. Adding `pnpm -r test` to `cargo make ci` would make the CompMD repository's one command check both; it needs Node and pnpm wherever CI runs.

## Plan

1. `validate/values.rs`: `Kind`, `Form`, the kinds, `accepts`, `describe` and `suggest`, with `grammar.rs`'s checks moved into it; `TypeName::kind` and `Universal::kind`; `refused` on both paths. Tests of each kind's message and suggestion, and the guard-rail tests over every kind.
2. The type `areas`, if agreed: the spec, `core.comp.md`, and the check by name removed.
3. `conformance/params.json` and its test; the spec test.
4. The renderer test against `params.json`, and whatever it finds missing in `styles.ts`.
5. `cargo make ci` green, with `pnpm -r test` in it if agreed.
