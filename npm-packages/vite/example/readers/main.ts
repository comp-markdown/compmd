// A page of readers' documents, with a configuration of its own (`compmd.config.json` beside it): the app's `stars`
// is strict here, so it runs in a frame, never in this page.
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "./compmd.ts";

const post = "---\nmd:\n  inputs:\n    rating: number = 4\n---\n\nA reader's post, rated :value[4]{of=(string(md.input.rating))} of 5.\n\n:: stars[4 of 5] {value=rating}\n";

await init();
const document_ = new OpenDocument({ ...request, path: "post.comp.md", files: { "post.comp.md": post }, now: new Date().toISOString() });
const view = createSandbox({ ...sandbox, page: "host" }).open(document.getElementById("page")!, { components: document_.components });
view.render(document_.expanded);
