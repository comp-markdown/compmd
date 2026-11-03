# 6. Data and inputs

A document reads data from files beside it, from URLs, and from your app itself, and its inputs are values a reader changes. The processor fetches none of it: your app gives it everything, so it decides what a document may reach. This chapter builds a page of orders that reads your app's own records, a file beside the document, and a status the reader picks, kept in the page's address.

How a document declares its data and inputs is [the Author's Guide](../authors/README.md)'s subject ([data from files](../authors/06-data-from-files.md), [inputs the reader changes](../authors/07-inputs-the-reader-changes.md)); here is what your app does with them.

## Data files

A data import's `from` is a relative path, a URL, or a `host:` source:

| `from` | What it reads | What your app gives |
| --- | --- | --- |
| `./delivery.csv`, `./team.yaml`, `./site.comp.md` | a file of the document's folder | the file, in `files` |
| `https://api.example.com/status.json` | a URL, from an origin you allow | its text, in `remote`, by URL |
| `host:orders?status=open` | your app's own data | the value, in `host`, by source |

Files are the simplest: they are in `files` like the document ([chapter 4](04-processing-documents.md)), and loaded by extension, `.csv` and `.tsv` as lists of records, `.yaml`, `.yml` and `.json` as their values, a `.comp.md` as its author's frontmatter fields. A file that isn't there, or doesn't parse, is F026; a record that doesn't match the import's `schema` is F027 and is dropped.

## The example

`orders.comp.md` reads the orders of one status from the app, and the delivery times from a file:

```md snippet=orders/orders.comp.md
---
md:
  inputs:
    status: open | shipped = open
  data:
    - from: host:orders?status={{md.input.status}}
      as: orders
      schema: {id: string, customer: string, total: number}
      fallback: ./sample-orders.json
    - from: ./delivery.csv
      as: delivery
      schema: {zone: string, days: number}
---

# Orders

:: select-input[Status: open] {bind=status}

::: each {in=(md.data.orders) as=o join=list}
- **{{o.id}}**, {{o.customer}}: {{o.total}} EUR

:: #empty
No orders.
::: /each

:value[2 orders]{of=(string(size(md.data.orders)) + " orders")} in all.

## Delivery

::: each {in=(md.data.delivery) as=d join=list}
- {{d.zone}}: {{d.days}} days
::: /each
```

[Try it](https://compmd.hakanserce.com/playground/?doc=guides/developers/orders) in the playground, which has no orders of yours: it shows the fallback.

`delivery.csv`:

```csv snippet=orders/delivery.csv
zone,days
Europe,2
Elsewhere,6
```

`sample-orders.json`, the fallback, for a host that doesn't provide `host:orders`:

```json snippet=orders/sample-orders.json
[
  { "id": "S-1", "customer": "A sample customer", "total": 10 }
]
```

The page, as in the quick start:

```html snippet=orders/index.html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <title>Orders</title>
  </head>
  <body>
    <main id="page" style="height: 28rem"></main>
    <script type="module" src="./main.ts"></script>
  </body>
</html>
```

And `main.ts`, with the same `vite.config.ts` as the quick start:

```ts snippet=orders/main.ts
import { init, OpenDocument, sources, type Request } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import delivery from "./delivery.csv?raw";
import text from "./orders.comp.md?raw";
import samples from "./sample-orders.json?raw";

await init();

/** The app's own records, which its documents read as `host:orders`. */
const ORDERS = [
  { id: "A-1041", customer: "Ada", total: 42.5, status: "open" },
  { id: "A-1042", customer: "Grace", total: 18, status: "open" },
  { id: "A-1039", customer: "Edsger", total: 7.25, status: "shipped" },
];

/** What the app gives a `host:` source, without `host:`; `undefined` for one it doesn't provide. */
function hostData(source: string): unknown {
  const [name, query = ""] = source.split("?");
  if (name !== "orders") return undefined;
  const status = new URLSearchParams(query).get("status");
  return ORDERS.filter((order) => order.status === status).map(({ id, customer, total }) => ({ id, customer, total }));
}

// The inputs' values, kept in the page's address: anyone can write one, and the processor checks each.
let inputs: Record<string, unknown> = Object.fromEntries(new URLSearchParams(location.search));

/** The document with the inputs' values, and the host data that render needs, which depends on them. */
function open(): OpenDocument {
  const files = { "orders.comp.md": text, "delivery.csv": delivery, "sample-orders.json": samples };
  const asked: Request = { ...request, path: "orders.comp.md", files, now: new Date().toISOString(), inputs };
  const host: Record<string, unknown> = {};
  for (const source of sources(asked).host) {
    const data = hostData(source);
    if (data !== undefined) host[source] = data;
  }
  return new OpenDocument({ ...asked, host });
}

let opened = open();
const view = createSandbox(sandbox).open(document.getElementById("page") as HTMLElement, { components: opened.components });
view.render(opened.expanded);

// The reader picked another status: keep it, and open the document again, with the data it now needs.
view.on("set", ({ input, value }) => {
  inputs = { ...inputs, [input]: value };
  history.replaceState(null, "", `?${new URLSearchParams(Object.entries(inputs).map(([name, given]) => [name, String(given)])).toString()}`);
  opened.free();
  opened = open();
  view.render(opened.expanded);
});
```

Run it: the page lists Ada's and Grace's orders, *2 orders in all*, and the two delivery zones. Pick *shipped*, and it lists Edsger's, and the address ends in `?status=shipped`, so a reload, or a link, opens the same view.

## The host's data

A `host:` source names data your app provides: the processor gives it nothing but the text after `host:`, here `orders?status=open`, and you decide what that means. It is the one way a document reads your app's own records, and it reads only what you answer.

- **`sources(request).host`** lists the sources a render needs, without `host:`, after interpolating the inputs. Since `from` may hold an input, the sources change with the inputs: that is why `open` asks again each time.
- **You answer in `request.host`**, by source, with any JSON value. It lands in `md.data` like a file's, checked against the import's `schema` (F027 drops a record that doesn't match), so give what the schema says, and only the fields it names.
- **A source you don't provide** is left out. The document then gets its `fallback`, with no diagnostic, or, without a fallback, `null` and F026: an `each` over it shows its `empty` slot.
- **Access is yours.** The document carries no credentials, and a reader sees only what your app answers for that reader. A shared document can ask for any source by name, so answer each as you would any request from that reader.

When your data changes, open the document again with the new data: rendering is a function of what the request holds ([spec §8.5](https://compmd.hakanserce.com/spec/#85-evaluation)).

## Data from URLs

A document may read an `https://` URL, but only from an origin you allow, in `allowedOrigins`. A document can't grant itself access: any other origin is F028, and the import gets its fallback.

```yaml
md:
  data:
    - from: https://status.example.org/status.json?region={{md.input.region}}
      as: status
      fallback: ./status.json
```

`sources(request).remote` lists the URLs a render needs from the allowed origins, with the inputs' values in them. Fetch them, and give each response as `{text, contentType}` in `remote`; the `Content-Type` decides the format, or else the URL's extension:

```ts
const asked = { ...request, path, files, now, allowedOrigins: ["https://status.example.org"] };
const remote: Record<string, Fetched> = {};
for (const url of sources(asked).remote) {
  const response = await fetch(url, { credentials: "omit", referrerPolicy: "no-referrer" });
  if (response.ok) remote[url] = { text: await response.text(), contentType: response.headers.get("content-type") };
}
const opened = new OpenDocument({ ...asked, remote });
```

A URL you couldn't fetch is left out, and the document shows its fallback, or F026 without one. What the spec asks of a host that fetches while a reader views a document ([spec §8.6](https://compmd.hakanserce.com/spec/#86-data-imports) and [§15](https://compmd.hakanserce.com/spec/#15-security)):

- **The reader decides.** The first time a document needs an origin the reader hasn't decided on, ask, and remember the answer. Until the reader allows it, leave it out of `allowedOrigins`. Origins your app configures need no question.
- **No credentials:** no cookies or other credentials with a document's fetches, except those you configured for that origin, and no `Referer`.
- **The origin learns** the reader's address, the time, and the input values in the URL. A shared cache, by URL, tells it less.

A document rendered once, at build time, reveals no reader: fetch there, and nothing reaches the origin from your readers.

## Inputs and their values

An input is a typed value the reader changes, with a default, so a document always renders. Its value comes from `request.inputs`, by name:

- **A value that doesn't fit is the default.** `?status=lost` opens the open orders, since `lost` isn't one of the enum's members; a number input given `"x"` gets its default. No diagnostic says so. So `inputs` can come from anywhere, a URL, storage, a reader's link, without checking it first.
- **The root of the expanded tree lists the inputs**, `expanded.inputs`, each with its `type`, an enum's `members`, and the `value` the render used. An inputs panel of your own, as the playground's, reads it; so does `frontmatterOf(request).inputs`.
- **Where values are kept between renders is yours.** The example keeps them in the page's address; another app keeps them per reader, or not at all.

## The components that set them

Two kinds of component change an input:

- **The input components**, built in: `date-input`, `number-input`, `text-input`, `select-input` for an enum, and `toggle-input` for a boolean. Each binds one input, `{bind=status}`, and renders a control.
- **A component in code** whose signature declares a param of type `input<…>`, which the document binds to an input (`:: map[Map] {selected=region}`). Its code calls `context.set(param, value)` ([chapter 10](10-your-apps-own-components.md)). An input such a component can change can't appear in a data import's `from` (F031), so component code can't send data out through a URL.

Either way, the sandbox tells your page with the view's `set` event, `{input, value}`, named by the input, not the param. It passes on only a `set` of one of the document's inputs; a frame that asks to set anything else is refused. In host mode, a trusted component's `set` comes the same way.

## Processing again with new values

A new value is a new render: open the document again with it, as the example does in its `set` listener. Rendering is a function of the request, so the page then shows what a fresh render with that value gives, the data included: here a new status is a new `host:` source, and `open` answers it before it processes.

The text is the same, so the new tree has the same origins, and `view.render` brings the page up to date in place: a component keeps its state across a new value. An editor that keeps its document open for edits does the same ([chapter 5](05-live-editing.md)). An open document that only needs data it fetched again, with the same inputs, takes `opened.reload({remote, host})` instead, which loads and expands again and gives a splice of the whole tree.

Next: [rendering and styling](07-rendering-and-styling.md), the page itself: its look, light and dark, and its links.
