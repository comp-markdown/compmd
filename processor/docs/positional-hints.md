# Positional hints

**Status: accepted, and built.**

A writer can't find the other values a param takes while the one written is valid: `border=subtle` is fine, so no diagnostic says what else `border` takes. Diagnostics help once something is wrong; a hint helps before. The processor answers one question, *what is at this place in the text?*, with structured data: a positional hint. The playground shows it in a line under the text, and a language server, later, turns the same hint into hovers, completions and "go to definition".

## Decisions

1. **One query, structured answers.** `Processor<Expanded>::hint_at(offset)` gives a `PositionalHint`: what is at a place in the processed document, a component, a param and its value, a slot, an interpolation's path, a name in an expression, or a frontmatter key, each with what it is, what it takes and where it is defined. It is data, not a sentence. The wording of a value's kind stays `Kind::describe()` (the one F009 uses), so a diagnostic and a hint never say different things.
2. **Completions come with the same hint**, `hint.completions`: the candidates for the text at the place, each with its label, the text it inserts and a few words on what it is; each replaces the hint's `range`. The lookup makes them from what it found, so nothing else lists a param's values or a component's params, and the binding sends them with the hint.
3. **It works on text being typed.** At the caret the text is often half-written (`border=`, `::: car`, `{{md.da`), and the tree has dropped what broke. So the query finds the block the caret is in by its position in the tree, which tells code, math and the frontmatter from text, and reads the caret's line again as the parser reads it, with its param-list reader, which keeps the params before a break. The component's name comes from the line, and what it stands for from the resolved names; a slot's container, and the names a template binds, come from the tree.
4. **A lookup, not a pass.** The query finds the top-level block at the offset (the blocks are in order, so a binary search), walks down to the node, reads one line, and looks up names in the resolved state. It never runs a stage again, so it can answer on every caret move, and an edit keeps it as cheap as the incremental state is.
5. **Positions are the processor's, in code points; the binding converts.** `hint_at` takes an offset in code points and gives ranges in lines and columns, as diagnostics do (spec §10.5). `OpenDocument.hintAt(offset)` takes the editor's UTF-16 offset, as `edit` does. A language server converts its `(line, character)` the same way, with the same line index.
6. **Diagnostics stay what they are, and the hint shows the one at the caret.** When the caret is in a diagnostic's range, the hint line shows that diagnostic first, with its fix as a button, then the hint. The list under the text doesn't change. A language server needs nothing of this: its client shows diagnostics in hovers by itself.
7. **In the playground, the hint comes with each answer from the worker.** The editor sends the caret's offset with each batch of edits, and alone when only the caret moved; the worker answers with the hint for the text it just processed, so the hint is never of an older text. No round trip of its own while typing.
8. **The hint line offers its values as buttons:** a param's value's tokens, and the type names on a type. Clicking one replaces what is written, as a diagnostic's fix does, through the same edit. Completion as you type (a list that opens on its own) waits for CodeMirror, and comes from `completions` then.
9. **All three phases in this work, built in order** (below): component lines and slots first, then interpolations and expressions, then frontmatter. The query's shape covers all of them from the start, so each phase adds answers, not a new API.
10. **The hints have their own module**, `compmd::hints`: `PositionalHint`, its subjects, the `…Hint` types, completions and the lookup itself. `Processor<Expanded>::hint_at` only hands the processor's state to `hints::at`.

## What the hint says, by where the caret is

Each example shows the text, with `|` at the caret, and the hint line the playground shows.

### On a component's name

```md
::: ca|rd {title="Setup"}
```

> **`card`**, a CompMD component defined in this document (line 53) · params `kind: note | warning = note`, `title: string` · slots `default`, `footer`

For a built-in: "**`box`**, built in (`@compmd/core`)". For a host-language component: "**`chart`**, JSX from `@acme/charts@1.4.2`" and its signature, or "no signature: its params go unchecked". For a name that stands for nothing, the diagnostic (F002, with its fix) and the names close to it.

```rust
Subject::Component { name, component: ComponentHint }
// ComponentHint: kind (compmd, processor, host language), source file, where it is defined, signature
```

The component part is `compmd::components::Listed`, already built for the Components tab, with where it is defined added.

### On a param's key

```md
::: box {padd|ing=m}
```

> **`padding`** on `box`, a common param · takes one or two of `none`, `s`, `m`, `l` or a length in `px`, `rem`, `em` or `%`, such as `s l` · takes a breakpoint (`md:padding=…`)

For a document's own component, its declaration: "**`tone`** on `badge` · `info | warning | danger`, default `info` · *the badge's color*", the last part being the declaration's `doc`. An unknown key shows F008 and its fix.

### On a param's value

```md
::: box {border=sub|tle}
```

> **`border`** on `box` · [`none`] [`subtle`] [`strong`] · or a CSS border, such as `1px solid #d0d7de`

The tokens are buttons; the one written is marked. This is the case the task started from. A value of a form only (`width=20rem`) shows the kind's description. An expression value (`border=(…)`) shows the param's kind, then the expression's hint (below).

### Between params, or after an `=` with nothing yet

```md
::: box {padding=m |}
::: box {border=|}
```

> `box` takes: `display`, `gap`, `align-items`, … and the common params `padding`, `border`, … (those already written left out)
>
> **`border`** on `box` · [`none`] [`subtle`] [`strong`] · or a CSS border

### On a slot line

```md
:: #foo|ter
```

> slot **`footer`** of `card` · `card` has slots `default`, `footer`

An unknown slot shows F006 and its fix; a template slot says what it iterates ("once per element of `rows`, as `t`").

### In an interpolation (second phase)

```md
{{md.data.sal|es}}
```

> **`md.data.sales`**, data from `./sales.csv` · a list of records: `month: string`, `region: string`, `units: number`

Each step of a path answers what it names: `md.doc.<key>` the frontmatter key and its value's type, `md.data.<name>` the import and its schema, `md.input.<name>` the input's type and value, a param in a definition its declaration, an `as` name its list, `md.loop` its fields. Completions list the names at that step.

### In an expression (second phase)

```md
::: if {when=(md.data.books.exi|sts(b, !b.read))}
```

> **`exists(x, predicate)`** on a list · true when any element satisfies the predicate

The same names as an interpolation, with their CEL types, and the profile's functions with their signatures and one line each (spec §8.2's tables, which then come from the processor's function table, with a test that the spec lists every function, as the vocabulary's tests do for params).

### In the frontmatter (third phase)

```yaml
md:
  inputs:
    since: num|ber = 1970
```

> type **`number`** · a number, such as `12` or `0.5` · the types: `string`, `number`, `boolean`, `date`, … or an enum `a | b`

The keys under `md` (`version`, `import`, `data`, `inputs`, …) with what each holds, the keys of a data import (`from`, `as`, `schema`, `cache`, `fallback`, `delimiter`), and type names in schemas and inputs.

## The API

### Processor

```rust
impl Processor<Expanded> {
    /// What is at `offset` (in code points) of the processed document's text, and what it takes; `None` where there
    /// is nothing to say, such as in plain prose.
    pub fn hint_at(&self, offset: usize) -> Option<PositionalHint>;
}
```

The rest is in `compmd::hints`:

```rust
/// What is at a place in the text.
#[derive(Serialize)]
pub struct PositionalHint {
    /// The text the hint is about: the name, the key, the value, the path step. A completion replaces it.
    pub range: Range,
    /// `None` where only a diagnostic has something to say.
    pub subject: Option<Subject>,
    /// The candidates for the text in `range`: tokens, keys, names, each with its label, its text and a detail.
    pub completions: Vec<Completion>,
    /// The diagnostics whose range holds the offset, as `diagnostics()` gives them.
    pub diagnostics: Vec<Diagnostic>,
}

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Subject {
    /// A component's name, on its line or inline.
    Component { name: String, component: Option<ComponentHint> },
    /// The param list, between params, or a key it doesn't take: the params the component takes, those written
    /// left out.
    Params { component: String, params: Vec<ParameterHint> },
    /// A param's key, or its value (`value` holds what is written).
    Param { component: String, param: ParameterHint, value: Option<String> },
    /// A slot line.
    Slot { component: String, slot: String, slots: Vec<SlotDeclaration> },
    /// A step of a path, in an interpolation or an expression (second phase).
    Path { path: String, name: NameHint },
    /// A function in an expression (second phase); `None` for a name that is no function.
    Function { name: String, hint: Option<FunctionHint> },
    /// A key in the frontmatter, or a value CompMD reads (third phase).
    Frontmatter { path: Vec<String>, about: Option<String> },
    /// A type name in an input's or a schema field's declaration (third phase).
    Type { name: String, takes: Option<KindHint> },
}

/// A param: what it is and what it takes.
#[derive(Serialize)]
pub struct ParameterHint {
    pub name: String,
    /// A common param, an attribute, or the component's own.
    pub origin: ParamOrigin,
    /// What it takes: `Kind`'s tokens, forms and examples, and `Kind::describe()`.
    pub takes: KindHint,
    pub required: bool,
    pub default: Option<String>,
    pub doc: Option<String>,
    pub breakpoints: bool,
}
```

- `ComponentHint` has what the Components tab's `Listed` has, but its uses, with the file that defines the component and where in it, so "go to definition" is one field.
- `KindHint` is `Kind`'s public fields as data, plus `description: Kind::describe()`, so no client words a kind itself. For a bound input (`input<number>`), its tokens are the inputs it can bind.
- `NameHint` holds a path step's type, what it is in words ("data from `./books.csv`") and a record's fields; `FunctionHint` each way to call a function and one line on what it gives. That line sits beside the function's overloads (`Library::doc`), and a test checks that spec §8.2 names every function.
- The frontmatter's keys are tables beside the code that reads them (`MD_KEYS`), or in the hints for the keys of imports, exports, data imports and declarations; a test checks that the spec's sections name every key.
- `compmd::hints` is `src/hints.rs` and holds only the hint code: the types above and the lookup, the component lines in `hints/invocation.rs`, paths and expressions in `hints/expression.rs`, the frontmatter in `hints/frontmatter.rs`. The components, expressions, frontmatter and vocabulary stay in their own modules; the hints read them, with the resolved state and `values`, and change nothing.

### Binding

```ts
class OpenDocument {
  /** What is at `offset` (UTF-16, as `edit` takes) of the text as the last change left it; `undefined` where there's nothing to say. */
  hintAt(offset: number): PositionalHint | undefined;
}
class OpenDocumentJson {
  hintAt(offset: number): PositionalHint | undefined; // the same, parsed: it is small
}
```

The types in `src/index.ts` mirror the Rust ones, as `Components` does.

### The playground

- **Worker protocol:** `{type: "open", request, caret}` and `{type: "edits", edits, caret}` (the caret after them), and `{type: "hint", caret}` when only the caret moved. The answer to either carries `hint`.
- **The hint line**, under the text, above the diagnostics: one line, cut with an ellipsis and opened in full on hover. Values are buttons that make the edit; the diagnostic at the caret, if any, comes first with its fix.
- **When it updates:** on `selectionchange` in the text area (a caret move by key or click). The session sends the caret's move as it comes when the worker is free, and keeps the latest for its next message while it is busy, so moves never queue. A selection shows the hint of its start. Waiting for an animation frame to send a move put every paint of the page a frame later (`pnpm bench`), so it doesn't.

## A language server, later

The server is a small Rust program over the processor crate. Each LSP request maps onto what this gives:

| LSP | From |
|---|---|
| `textDocument/hover` | `hint_at(offset)`, written as Markdown: the subject's line, then `takes.description`, the default and the `doc` |
| `textDocument/completion` | `hint_at(offset).completions`; each `Completion`'s text, with the hint's range, is a `TextEdit` |
| `textDocument/definition` | `Subject::Component`'s definition range and file |
| `textDocument/signatureHelp` | `Subject::Params`/`Param`: the component's params, the one at the caret active |
| `textDocument/publishDiagnostics` | `diagnostics()`, as today; `fix` becomes a code action |
| positions | `(line, character)` in UTF-16 → code points, with the binding's line index (`wasm/src/text.rs`), which moves into the processor crate when the server is built |

A tree-sitter grammar is beside this: it highlights syntax and knows nothing of what a name stands for.

## Phases

| Phase | Answers | Where it shows |
|---|---|---|
| 1 | component names, param keys and values (written, empty, half-typed), the param list between params, slot lines; the diagnostic at the caret | the hint line, with value buttons |
| 2 | interpolation paths and expressions: names, their types, the profile's functions | the hint line |
| 3 | frontmatter: `md` keys, data import keys, type names | the hint line |
| later | CodeMirror's completions and hovers; a language server | both from `hint_at` and `completions` |

## Tests

- **Processor** (`src/hints.rs` and `tests/hints.rs`): each subject at its place, for a built-in, a document's own component and a host-language one; half-typed lines (`border=`, `{padding=m `, a container opener without its closer); a place in prose gives `None`; after an edit, the hint of the new text. A test that every `Kind` in `Kind::ALL` gives completions that it accepts.
- **Binding:** UTF-16 offsets after an emoji land on the same subject as code points do.
- **Editor:** the worker answers edits with the hint of the text it processed; clicking a token makes the edit; the line in Chrome, light and dark.

## Not in this change

- A completion list as you type, which waits for CodeMirror.
- The language server and its line index.
