# 21. The diagnostics

Every diagnostic in the spec's [catalogue](https://compmd.hakanserce.com/spec/#102-catalog), as the packages report it: its code, its severity, the stage that finds it, what causes it, who causes it, and what fixes it. Writers have their own appendix, with an example of each: [the diagnostics and their fixes](../authors/20-the-diagnostics-and-their-fixes.md) in the Author's Guide. This one is for the developer whose app shows them, or whose pack causes them.

## Reading a diagnostic

A diagnostic is the spec's [machine form](https://compmd.hakanserce.com/spec/#105-machine-format), as `process`, `OpenDocument` and the crate give it:

| Field | What it is |
|---|---|
| `code` | `F…` for an error, `W…` for a warning |
| `severity` | `error` for an F code, `warning` for a W code |
| `message` | what is wrong, for people; not part of the spec, and it may change |
| `file` | the document it is in, when it isn't the one processed: a path, or `@scope/pack@version/path` for a pack's |
| `range` | where: `start` and maybe `end`, lines and columns from 1, columns in code points |
| `related` | other places, innermost first: an unclosed container's opener, the invocation that led into a pack |
| `stage` | `parse`, `resolve`, `validate`, `load` or `evaluate` |
| `fix` | an edit that fixes it, when the message suggests one: the text at a range on one line becomes `text` |

An editor turns a diagnostic into an LSP diagnostic and its `fix` into a quick fix; columns are code points, so convert them to UTF-16 for an editor that counts that way. Errors don't stop processing: the processor recovers as the spec says ([section 10.3](https://compmd.hakanserce.com/spec/#103-recovery-rules-lenient-mode)) and gives the whole tree, fallbacks where something failed. The spec calls that lenient mode, for previews; strict mode, for CI and publishing, fails on any F code: the request's `mode: "strict"` gives the same diagnostics and tree, with `failed` set ([the Rust crate](19-the-rust-crate.md#checking-documents-in-ci) has a check that uses it).

**Who causes it**, in the tables: a **writer**, in the document's text; a **pack author**, in a pack's files or `package.json`, which the document's writer can't fix but can work around (another version, another pack); a **host**, the app, by its configuration or by what it gives the processor. A writer can cause a pack-author code by writing a pack.

The guides' tests make each code below with a small document and check the processor reports it at the stage this appendix gives.

## Stage 1, `parse`

The document's text alone: no imports, signatures or data.

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| F001 | error | `md.version` names a major version the processor doesn't implement; the file is read as plain CommonMark and GFM | writer | `version: 1`, or a processor that implements it |
| F003 | error | a closer names no open container while one is open, usually a misspelt name | writer | the closer's name, `::: /box` |
| F004 | error | a container isn't closed before an outer closer or the end of the file | writer | its closer, in the same list item or quote |
| F005 | error | a closer while no container is open | writer | removing it, or the opener it was meant for |
| F007 | error | a named slot twice in one container | writer | one slot line per slot |
| F011 | error | a broken param list: an unterminated string, an unbalanced bracket, invalid JSON, a bare value with `(` or `{`, an empty value; also on a rendered code block's info string, found in validation | writer | the quote or bracket; quoting the value, or an expression `(…)` |
| F012 | error | a continuation line, `::: + …`, not right after an opener, a leaf or another continuation | writer | moving it under its opener |
| F015 | error | a slot line outside any container | writer | moving it into its container |
| F016 | error | a line that starts with `::` but is no CompMD form, such as `:::box` or `::: @kit/card` | writer | the form the message names, `::: box` |
| F017 | error | a param given twice, `id` with `#name`, or a code renderer's `code` on a code block's info string | writer | one of them |
| F032 | error | a closer names a container opened outside its list item or quote | writer | the closer at the container's own level |
| F033 | error | frontmatter that isn't YAML 1.2 core: tags, anchors, aliases or a repeated key; the file is read as plain CommonMark and GFM | writer | plain YAML |
| F036 | error | a param list on a signature fence, which is ignored | writer, pack author | moving the params to the definition fence |
| W101 | warning | a leaf without fallback text, or a rendered code block without `alt`, found in validation | writer | `[the sentence it stands for]`, or `alt="…"` |
| W106 | warning | `{{` followed by something that isn't a path; kept as text | writer | a path, `\{{` for literal braces, or `:value` for a computed value |
| W107 | warning | an inline component, a space, then a param list, which is text | writer | the param list right after `]` |
| W108 | warning | a container opener or leaf right after a paragraph line | writer | a blank line before it |
| W110 | warning | a later minor version than the processor implements | writer, host | a newer processor, or an earlier version |
| W111 | warning | an unknown key under `md`, or an unknown name in `md.syntax`; ignored | writer | the key the message suggests |
| W116 | warning | a line that starts with `::` inside an HTML block, which makes it HTML | writer | a blank line after the HTML's opening line |

## Stage 2, `resolve`

The imports, packs and names. What a pack's files cause is reported in the pack's file (`@scope/pack@version/path`), with the importing document's invocation as `related`.

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| F002 | error | an unknown component name, or a signature fence with nothing to attach to; the message suggests the nearest name | writer | the name, or the import that gives it |
| F013 | error | a name declared twice, a keyword (`if`, `each`, `value`, `box`) defined, imported or renamed to, `md` declared, a CEL reserved word or type name as a param, `as` name or input, or two params differing only in `-` and `_`; found in validation for names in a body | writer, pack author | another name |
| F019 | error | a component whose definition invokes itself, directly, through others, or through a code block it renders | writer, pack author | breaking the cycle |
| F020 | error | an import cycle | writer, pack author | removing the import that closes it |
| F021 | error | an import names something its source doesn't export | writer | the name the source exports, or `"*"` |
| F022 | error | a pack's `export.pack` or `export.version` disagrees with its `package.json` | pack author | the two the same; `compmd-pack build` checks it |
| F023 | error | a name, or a code block's language, that two wildcard imports, or one and `@compmd/core`, give is used; for a code block's language found in validation | writer | listing the name in one import |
| F026 | error | an import that can't be loaded: a missing file, no release matching the version, an unscoped name, `@compmd/*` other than core, a path leaving its root, or a pack the host's policy doesn't allow, denies, or left out (its size); for data, found when loading. Never for a pack or data source the request names as `pending`: it waits for the host to give up | writer, host | the path or version; for a pack the host refuses, the host's configuration, or another pack |
| F029 | error | an exported host-language component without a signature fence | pack author | its signature fence |
| F035 | error | a pack whose release doesn't match the import's `integrity`, or that the host found broken | writer, host | the integrity npm publishes for that version, or the host giving the release's real integrity |
| F037 | error | a declaration's type isn't in the type grammar, in a signature, an input or a data `schema`; it takes any value | writer, pack author | the type the message suggests |
| W105 | warning | a local host-language component, or an imported component file (`./Chart.jsx`), without a signature fence, whose params can't be typed | writer | a signature fence |
| W112 | warning | a local definition or explicit import shadows a `@compmd/core` component | writer | another name, if it wasn't meant |
| W113 | warning | a name from an ambient pack is used, when the host asks for it (`reportAmbient`); for a `host:` data source, found when loading | host | nothing: it tells a writer the document isn't portable |
| W117 | warning | an exported host-language component without `runtime=` | pack author | `runtime=react@19` or `svelte@5` on its fence |
| W121 | warning | a pack's `peerDependencies` names a runtime or library the host doesn't have in that range, or a library pack that can't be had (see also rendering, below) | pack author, host | the host offering it, a pack range it meets, or the library pack published |

## Stage 3, `validate`

Every use against its signature, the paths, the expressions' types, the inputs and the data sources.

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| F006 | error | a slot its component doesn't declare, whose content joins the default slot; or content before the first slot line of a component without a default slot, which follows the component | writer | a declared slot |
| F008 | error | an unknown param, or a breakpoint on a param that takes none | writer | the param the message suggests |
| F009 | error | a param value that doesn't match its type or grammar, or a `grid-area` naming no area of its box's literal `grid-template-areas`, and, found in resolving, a code renderer's `code` declared as anything but a required `string`, or an import's `integrity` on a range instead of an exact version | writer, pack author | a value of the type, which the message lists |
| F010 | error | a required param missing, or an input without a default, which then starts empty | writer | the param, or the input's default |
| F014 | error | an unknown interpolation path | writer | the path, or `\{{` for literal braces |
| F018 | error | a body given to a component that declares no slots | writer | the component as a leaf |
| F028 | error | remote data from an origin the host doesn't allow (`allowedOrigins`), or that the reader hasn't allowed yet (`loadData`) | writer, host | the host or the reader allowing the origin, or a `fallback` |
| F030 | error | a bound param or input component's `bind` names an undeclared input, or one of another type | writer | the input declared, of that type |
| F031 | error | a `from` that interpolates something other than `md.input` and `md.doc`, an input a component can set, or a URL's scheme or host | writer | literal scheme and host; values in the path and query only |
| W102 | warning | a use of a component whose definition language or runtime the processor doesn't run (`vue`, or a runtime the host's `policy.runtimes` doesn't have); its fallback is shown | writer, pack author, host | a supported language and runtime, or the runtime in the frames |
| W103 | warning | an `each` whose default slot is one list item, with `join` unset | writer | `join=list` |
| W120 | warning | a box param given to a `display=contents` component, which has no box; ignored | writer, pack author | removing it |

## Stage 4a, `load`

The inputs' values and the data.

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| F027 | error | a data record that violates its `schema`, or a CSV row with the wrong number of cells; the record is dropped | writer | the data, or the schema |

F026 for data is found here too: a file that is missing or unparseable, a `host:` source the host doesn't give, a CSV header with an empty or repeated name.

## Stage 4b, `evaluate`

Expanding the tree, for one render context.

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| F024 | error | an expression's parse, type or evaluation error, or a list or a map without `name` written into text; type errors are found in validation | writer | the expression; a `schema` for the data it reads |
| F025 | error | an `each` iteration that isn't the shape its `join` needs | writer | the iteration's content, or another `join` |
| F034 | error | an expansion larger than the processor allows, past 100,000 nodes | writer | less nesting, or filtering the data |
| W109 | warning | an interpolated link or image URL with a scheme other than `https:`, `http:` or `mailto:`; shown without the link | writer | the data's URLs |
| W114 | warning | a box whose blocks use breakpoints, with no `width` where its width would come from its content | writer, pack author | a `width` on the box |
| W119 | warning | a box that places blocks by position, with no `height`, `aspect-ratio` or `fill` at some width | writer, pack author | one of them on the box |

## When rendering

What only running a component finds. `@compmd/sandbox` reports it in the view's `error` event, `{ component, reason, code?, message }`, with `code` from `diagnosticOf(reason)`:

| `reason` | `code` | What happened |
|---|---|---|
| `threw` | W118 | the component threw while mounting or updating, called `fail`, or its frame stopped answering |
| `failed` | W121 | its code didn't compile or load: an import it may not make, a compile error, a built module that doesn't match its hash |
| `missing` | none | the request gave no code for it; the processor reported why |
| `not-here` | none | a component of a level its place can't hold: a trusted or permissive one in frame mode, or in a strict component's slot; this is the sandbox's design |

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| W118 | warning | a host-language component threw, failed or hung; its fallback is shown | pack author | the component; a test with `mountForTest` ([chapter 17](17-testing-and-publishing.md)) |

W121 at rendering is the same code as above: the sandbox reports a module that can't load, and the processor a pack's `peerDependencies`.

## Only in the Rust crate

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| W115 | warning | the host gave metadata for a document that has its own frontmatter, which wins; at line 1, in parsing | host | giving metadata only for documents without frontmatter |

The crate takes a document's metadata apart from its text (`ParseOptions::metadata`); `@compmd/processor`'s request has no field for it, so a host on WebAssembly can't give metadata and never sees W115.

## In the catalogue, not reported

| Code | Severity | What causes it | Who | What fixes it |
|---|---|---|---|---|
| W104 | warning | a signature fence and its component disagree: a prop, type or slot | pack author | the fence and the component the same |

W104 needs the spec's *component analysis* feature, reading a component's code for its props, which no package implements yet: a signature fence is believed as it is written. W113 for a library the host offers, which the spec allows, isn't reported either.

Next: [a glossary](22-a-glossary.md).
