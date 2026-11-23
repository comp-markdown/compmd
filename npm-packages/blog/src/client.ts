// The script of a post's page that has host-language components (docs/blog-tool.md, decision 5): the page shows the
// post as it was rendered at build time, with the components' fallbacks; this renders it again through the sandbox,
// with the blog's packs from `@compmd/vite`, out of sight, and puts it in the static post's place once its components
// have drawn. Vite builds it, in the blog's own build.

/// <reference types="@compmd/vite/client" />
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import { POST_DATA, POST_FOLDER } from "./client-data.ts";
import { rebasedTree } from "./rebase.ts";

/** A live post's data: its document and files, and the origins it reaches without asking a reader, as it was built. */
interface PostData {
  path: string;
  files: Record<string, string>;
  origins: string[] | "any";
}

/** Renders the post live, in place of `body`, the static post. */
async function live(body: HTMLElement, { path, files, origins }: PostData): Promise<void> {
  await init();
  let inputs: Record<string, unknown> = {};
  /** What the reader set in each example (spec §7.8), by its number: the example's own, never the post's inputs. */
  let exampleInputs: Record<number, Record<string, unknown>> = {};
  const open = () => new OpenDocument({ ...request, path, files, allowedOrigins: origins, now: new Date().toISOString(), inputs, exampleInputs });
  let document_ = open();
  // Laid out, so the components size themselves, but out of sight until they have drawn.
  const target = document.createElement("div");
  target.className = body.className;
  target.style.cssText = "position:absolute;visibility:hidden;inset-inline:0";
  body.after(target);
  // The images of the origins the blog reaches are never asked about, as the build showed them (spec §15).
  const view = createSandbox({ ...sandbox, page: "host", origins: { configured: origins } }).open(target, { components: document_.components });
  let rendered = false;
  let loaded = false;
  const show = () => {
    if (!rendered || !loaded) return;
    target.style.cssText = "";
    body.remove();
  };
  view.on("loaded", () => {
    loaded = true;
    show();
  });
  view.on("rendered", () => {
    rendered = true;
    show();
  });
  // The page is a folder below the posts', where the post's relative URLs start, as the build rebased them.
  const render = () => {
    view.render(rebasedTree(document_.expanded, POST_FOLDER));
  };
  const reopen = () => {
    document_.free();
    document_ = open();
    render();
  };
  view.on("set", ({ input, value }) => {
    inputs = { ...inputs, [input]: value };
    reopen();
  });
  view.on("exampleSet", ({ example, input, value }) => {
    exampleInputs = { ...exampleInputs, [example]: { ...exampleInputs[example], [input]: value } };
    reopen();
  });
  render();
}

const data = document.getElementById(POST_DATA);
const body = document.querySelector<HTMLElement>(".post-body");
if (data?.textContent && body) void live(body, JSON.parse(data.textContent) as PostData);
