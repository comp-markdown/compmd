import { init, OpenDocument } from "@compmd/processor";
import { defaultCache, resolvePacks } from "@compmd/registry";
import { createSandbox } from "@compmd/sandbox";
import { registry, request, sandbox } from "virtual:compmd";
import text from "./plan.comp.md?raw";

await init();

// The document with what the app gives it, then the packs it imports that the app doesn't install, from the registry.
const asked = { ...request, path: "plan.comp.md", files: { "plan.comp.md": text }, now: new Date().toISOString() };
const resolved = registry && (await resolvePacks(asked, { ...registry, cache: defaultCache() }));
for (const { pack, message } of resolved?.problems ?? []) console.warn(`${pack}: ${message}`);
const opened = new OpenDocument(resolved?.request ?? asked);

// A fetched pack's built module reaches its frame as code, already checked; an installed one's by its URL.
const page = document.getElementById("page") as HTMLElement;
const modules = (place: Parameters<typeof sandbox.modules>[0]) => resolved?.modules(place) ?? sandbox.modules(place);
const view = createSandbox({ ...sandbox, modules }).open(page, { components: opened.components });
view.render(opened.expanded);
