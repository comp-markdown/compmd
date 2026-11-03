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
