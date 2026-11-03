# 4. Processing documents

`process` takes a document with everything it needs and gives back its trees and its diagnostics. This chapter goes through its request field by field, the files a document reads, what comes back, and what to do with a diagnostic and its fix. It builds a script that checks a document as a build would, run with Node.

## Loading the processor

`@compmd/processor` is WebAssembly. Load it once, before the first call:

- **In a browser**, `await init()` fetches the module next to the package's JavaScript, as the quick start does. A bundler such as Vite serves it.
- **In Node**, give the module's bytes to `initSync`: `initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) })`. The package exports the file by that name.

After that, every call is synchronous: `process`, `sources`, `frontmatterOf`, `componentsOf`, `plainHtml`, and the open documents of [chapter 5](05-live-editing.md).

## The request

A request is a plain object. Only `path` and `now` are required:

| Field | What it is |
| --- | --- |
| `path` | the processed document's path in `files` |
| `files` | the files of the document's root folder, by `/`-separated path: the document, the documents it imports, its data |
| `packs` | the packs a document may import, by name, each a list of releases: `{version, integrity?, broken?, files}` |
| `ambient` | packs every document gets without importing them, at the lowest priority |
| `reportAmbient` | report each use of a name from an ambient pack, W113 |
| `policy` | your policy for packs: their trust, whether a document may import packs you don't list, those you deny or left out |
| `allowedOrigins` | the origins remote data may come from, such as `https://status.example.org` |
| `remote` | remote data you fetched beforehand, by URL: `{text, contentType?}` |
| `host` | your app's own data, by source without `host:` |
| `now` | the render time, RFC 3339: what `md.now()` and `md.today()` give |
| `tz` | the time zone for a document without a `tz` key: an IANA name, or `UTC` (the default) |
| `inputs` | the values given for the document's inputs, by name |
| `origins` | whether expanded nodes carry their origins, as a live preview needs |

In a Vite app, `virtual:compmd`'s `request` holds `packs`, `ambient` and `policy` from your configuration ([chapter 9](09-packs-from-npm.md)); you spread it in and add the rest. The processor reads nothing else: no file system, no network, no clock. The same request always gives the same result, which is what lets you check a document in CI and get what a reader will see.

`process` throws only when the request itself is wrong: `now` isn't RFC 3339, or `path` isn't one of `files`. Everything wrong with the document is a diagnostic.

## The files a document reads

A document reads files by relative path: the documents it imports (`from: ./parts/notice.comp.md`) and its data (`from: ./releases.csv`). Each path resolves against the file it is written in, inside the root folder `files` stands for, and a path that leaves it (`../`) is F026. So you decide what a document may read by what you put in `files`.

The example is a folder with a document, the data it reads beside it, and a document it imports in a subfolder:

```md snippet=check-docs/quarter.comp.md
---
title: Releases this quarter
md:
  import:
    - from: ./parts/notice.comp.md
      names: [notice]
  data:
    - from: ./releases.csv
      as: releases
      schema: {version: string, date: string, changes: number}
---

# {{md.doc.title}}

::: notice {title="Three releases" border=subtle}
:value[24 changes]{of=(md.number(md.sum(md.data.releases.map(r, r.changes)), 0) + " changes")} in all, each listed below.
::: /notice

::: each {in=(md.data.releases) as=r join=list}
- **{{r.version}}**, {{r.date}}: {{r.changes}} changes
::: /each
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/check-docs) in the playground.

Its data, `releases.csv`:

```csv snippet=check-docs/releases.csv
version,date,changes
2.0,2026-07-14,9
2.1,2026-08-25,11
2.2,2026-10-06,4
```

And `parts/notice.comp.md`, which defines the `notice` component in CompMD and exports it:

````md snippet=check-docs/parts/notice.comp.md
---
md:
  export:
    names: [notice]
---

A box with a title, for what a reader should see first.

```compmd component=notice {padding=m border-radius=m background=subtle}
params:
  title: string
slots: [default]
---
**{{title}}**

{{md.slot}}
```
````

Packs are not files of the folder: a document imports them by name (`from: "@kit/charts@1"`), and a pack is in the request only if `packs` holds a release of it. Each release's `files` are the pack's own: its `package.json`, its document, what that imports. A built module's text can be left empty, since the processor only names it for the renderer. [Chapter 9](09-packs-from-npm.md) has where packs come from.

## The script

`check.ts` reads a folder's files, asks what the document needs fetched, processes it, and prints each diagnostic with its fix:

```ts snippet=check-docs/check.ts
// Checks a CompMD document as a build would: what it needs fetched, every diagnostic with its fix, and whether any is
// an error. Run it with Node: `node check.ts <folder> <document>`, such as `node check.ts . quarter.comp.md`.
import { readdirSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join, relative, sep } from "node:path";
import { argv, exit } from "node:process";
import { initSync, process as processDocument, sources, type Diagnostic, type Request } from "@compmd/processor";

// In Node the module is loaded from its file; in a browser, `await init()` fetches it.
initSync({ module: readFileSync(createRequire(import.meta.url).resolve("@compmd/processor/compmd_wasm_bg.wasm")) });

/** The text files under `folder` a document may read, by their `/`-separated paths in it. */
export function filesOf(folder: string): Record<string, string> {
  return Object.fromEntries(
    readdirSync(folder, { recursive: true, withFileTypes: true })
      .filter((entry) => entry.isFile() && /\.(comp\.md|csv|tsv|json|ya?ml)$/.test(entry.name))
      .map((entry) => {
        const path = join(entry.parentPath, entry.name);
        return [relative(folder, path).split(sep).join("/"), readFileSync(path, "utf8")];
      }),
  );
}

/** A diagnostic as a line: where it is, its code and message, and the edit that fixes it, when it has one. */
export function lineOf(document: string, { file, range, code, message, fix }: Diagnostic): string {
  const at = `${file ?? document}:${String(range.start.line)}:${String(range.start.column)}`;
  const fixed = fix ? `\n  fix: write \`${fix.text}\` at ${String(fix.range.start.line)}:${String(fix.range.start.column)}-${String(fix.range.end.column)}` : "";
  return `${at} ${code} ${message}${fixed}`;
}

/** Checks `document`, one of `folder`'s files, rendered at `now`: the lines to print, and whether it has an error. */
export function check(folder: string, document: string, now: string): { lines: string[]; failed: boolean } {
  const request: Request = { path: document, files: filesOf(folder), now };
  const { remote, host } = sources(request);
  const needed = [...remote, ...host.map((source) => `host:${source}`)].map((source) => `${document}: needs ${source}, which a check doesn't fetch`);
  const { diagnostics } = processDocument(request);
  return { lines: [...needed, ...diagnostics.map((diagnostic) => lineOf(document, diagnostic))], failed: diagnostics.some(({ severity }) => severity === "error") };
}

if (import.meta.main) {
  const [folder = ".", document = "index.comp.md"] = argv.slice(2);
  const { lines, failed } = check(folder, document, new Date().toISOString());
  console.log(lines.length === 0 ? `${document}: no diagnostics` : lines.join("\n"));
  if (failed) exit(1);
}
```

`process` is imported as `processDocument`, since Node has a `process` of its own. Install `@compmd/processor`, and run it with Node 24.2 or later, which runs TypeScript as it is and has `import.meta.main`:

```sh
npm install @compmd/processor
node check.ts . quarter.comp.md
```

It prints `quarter.comp.md: no diagnostics`.

## The result

`process(request)` gives three things:

- **`parse`**, the parse tree: the document as written, mdast with CompMD's own nodes ([spec §12](https://compmd.hakanserce.com/spec/#12-document-tree)). Tools that read or rewrite documents use it.
- **`expanded`**, the expanded tree: content only, with definitions, `if`, `each` and interpolations replaced by what they give. It is what a page renders ([chapter 7](07-rendering-and-styling.md)), with the inputs and their values on its root.
- **`diagnostics`**, of every stage, in stage order.

`process` doesn't give the components the tree names with their code; an open document does (`OpenDocument.components`), which is why the quick start uses one even for a page that never changes.

## Diagnostics and their fixes

Change `border=subtle` to `border=subtl` in `quarter.comp.md`, and run the script again:

```text
quarter.comp.md:15:43 F009 `border` takes `none`, `subtle`, `strong` or a CSS border, such as `1px solid #d0d7de`, so the param is treated as absent; did you mean `subtle`?
  fix: write `subtle` at 15:43-48
```

It exits with 1, so a CI job that runs it fails. A diagnostic has:

| Field | What it is |
| --- | --- |
| `code` | `F…` for an error, `W…` for a warning ([spec §10.2](https://compmd.hakanserce.com/spec/#102-catalog)) |
| `severity` | `error` or `warning` |
| `message` | what is wrong, in words, for people |
| `file` | the file it is in, when it isn't the processed document: an imported document, or `@scope/pack@version/path` for a pack's |
| `range` | where: `start` and, mostly, `end`, each a 1-based `line` and `column` |
| `related` | other places that matter, each with its `file`, `range` and `message`, innermost first: the invocations that led to an error in a definition, say |
| `stage` | `parse`, `resolve`, `validate`, `load` or `evaluate` |
| `fix` | when the message suggests an edit: the text at `fix.range`, on one line, becomes `fix.text` |

Columns count code points, not UTF-16 units: in the line `😀 Hi`, `Hi` is at column 3, while JavaScript counts the emoji as two units and puts `Hi` at index 3, which would be column 4. To make a fix in a JavaScript string, count the line's characters as code points; [chapter 5](05-live-editing.md)'s editor does, in `offsetOf`.

The processor always recovers. With an error, it drops the smallest broken piece, a param, a line, a record, and goes on, so there is always a page: here the box has no border. It never hides author content ([spec §10.3](https://compmd.hakanserce.com/spec/#103-recovery-rules-lenient-mode)). What an error means is yours to decide: a build fails on any `F`, as the script does, and a preview shows the page with the diagnostics beside it.

## What a document needs fetched

The processor fetches nothing. `sources(request)` says what a render of the document needs that only you can get:

- `remote`: the URLs of its remote data, from origins in `allowedOrigins`;
- `host`: its `host:` sources, without `host:`;
- `packs`: the packs it imports that `packs` has no matching release of, each `{pack, range, integrity?}`.

Each comes once, in the order the document names it, and it lists what the render needs, whether or not the request already holds it: compare with what you have. You fetch, put what you got in `remote`, `host` or `packs`, and process. A source you couldn't get stays out of the request, and the document shows its `fallback`, or F026. [Chapter 6](06-data-and-inputs.md) does this for data, and `@compmd/registry` for packs ([chapter 9](09-packs-from-npm.md)). The script only reports what it finds, since a check fetches nothing.

## Other questions to ask

Two more functions take the same request:

- **`frontmatterOf(request)`**: what the frontmatter gives the render: the author's own fields (`doc`), the settings under `md`, each data import with what was loaded (`null` when it failed), and the inputs with their values. An app's inputs panel or a document's properties come from it.
- **`componentsOf(request)`**: the components the document can name, grouped by the file that defines them, each with its kind, its signature and where the document uses it; and the names it uses that stand for no component. A list of a document's components, or a "go to definition", comes from it.

[The processor's README](../../wasm/README.md) has every type.

## The processor's limits

The processor bounds what a document can make it do, whoever wrote it:

- **An expanded tree of 100,000 nodes.** A document whose `each` or template slot would make more gets F034 at the one that crossed the limit, which shows its fallback; what came before stays ([spec §8.5](https://compmd.hakanserce.com/spec/#85-evaluation)). Four nested lists of 100 records are 100 million nodes, so the limit is what keeps such a document from hanging a page.
- **100,000 steps per expression**, a step for each node evaluated and each character or element it builds. An expression that takes more is F024, "the expression takes too much work". A filter takes about six steps a record.

Neither can be changed in `@compmd/processor`.

How long processing takes is measured in a worker in Chrome and WebKit, against budgets: a 4 KB document in 14 ms, a 52 KB one in 48 ms, an edit to an open document in 1 ms ([the README's budgets](../../wasm/README.md#budgets)). On an Apple M4, `process` takes 2 to 10 ms for these, and an edit 0.1 to 0.3 ms.

## In a worker

Processing a long document takes milliseconds that a page's main thread may not have while a reader types or scrolls. `process` runs the same in a worker, and its request and result are plain data, which `postMessage` copies:

```ts
// processing.worker.ts
import { init, process, type Request } from "@compmd/processor";

const ready = init();
addEventListener("message", async ({ data }: MessageEvent<Request>) => {
  await ready;
  postMessage(process(data));
});
```

The page makes it with `new Worker(new URL("./processing.worker.ts", import.meta.url), { type: "module" })`, posts each request, and renders what comes back. For an editor, which processes on every key press, a worker keeps an open document instead, and posts only what changed: [chapter 5](05-live-editing.md) shows how the playground does it.

Next: [live editing](05-live-editing.md), a page that follows its text as it is typed.
