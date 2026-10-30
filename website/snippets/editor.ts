import { init, OpenDocument } from "@compmd/processor";
import { createPage } from "@compmd/render";

await init();

const open = new OpenDocument({ path: "notes.comp.md", files: { "notes.comp.md": "# Hello world\n" }, now: new Date().toISOString() });
const page = createPage(document.body);
page.render(open.expanded);

// The reader selected `world` and typed `there`: only the heading is processed and rendered again.
page.splice([open.edit(8, 13, "there").splice]);
