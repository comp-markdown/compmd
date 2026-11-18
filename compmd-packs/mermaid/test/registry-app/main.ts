// A host that installs no pack: the document's import of the Mermaid pack is resolved from the registry its
// configuration names, through its own proxy, and the pack's module reaches the frame as code.
import { init, OpenDocument } from "@compmd/processor";
import { defaultCache, resolvePacks } from "@compmd/registry";
import { createSandbox } from "@compmd/sandbox";
import { registry, request, sandbox } from "virtual:compmd";

const DOCUMENT = `---
md:
  import:
    - from: "@comp-markdown/mermaid@0"
      names: [mermaid]
---

\`\`\`mermaid {alt="Fetched, then drawn"}
graph LR
  Fetched --> Drawn
\`\`\`
`;

await init();
const asked = { ...request, path: "doc.comp.md", files: { "doc.comp.md": DOCUMENT }, now: new Date().toISOString() };
const resolved = registry && (await resolvePacks(asked, { ...registry, cache: defaultCache() }));
const opened = new OpenDocument(resolved?.request ?? asked);
const page = document.getElementById("page");
if (page) {
  const view = createSandbox({ ...sandbox, page: "host", modules: (place) => resolved?.modules(place) ?? sandbox.modules(place) }).open(page, { components: opened.components });
  view.render(opened.expanded);
}
document.getElementById("diagnostics")?.replaceChildren(
  ...[...opened.diagnostics.map(({ code, message }) => `${code}: ${message}`), ...(resolved?.problems ?? []).map(({ pack, message }) => `${pack}: ${message}`)].map((text) => {
    const item = document.createElement("li");
    item.textContent = text;
    return item;
  }),
);
