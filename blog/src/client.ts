// The script of a post's page that has host-language components (docs/blog-tool.md, decision 5): the page shows the
// post as it was rendered at build time, with the components' fallbacks; this renders it again through the sandbox,
// with the blog's packs from `@compmd/vite`, out of sight, and puts it in the static post's place once its components
// have drawn. Vite builds it, in the blog's own build.

/// <reference types="@compmd/vite/client" />
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";
import { POST_DATA } from "./client-data.ts";

/** Renders the post live, in place of `body`, the static post. */
async function live(body: HTMLElement, { path, files }: { path: string; files: Record<string, string> }): Promise<void> {
  await init();
  let inputs: Record<string, unknown> = {};
  const open = () => new OpenDocument({ ...request, path, files, now: new Date().toISOString(), inputs });
  let document_ = open();
  // Laid out, so the components size themselves, but out of sight until they have drawn.
  const target = document.createElement("div");
  target.className = body.className;
  target.style.cssText = "position:absolute;visibility:hidden;inset-inline:0";
  body.after(target);
  const view = createSandbox({ ...sandbox, page: "host" }).open(target, { components: document_.components });
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
  view.on("set", ({ input, value }) => {
    inputs = { ...inputs, [input]: value };
    document_.free();
    document_ = open();
    view.render(document_.expanded);
  });
  view.render(document_.expanded);
}

const data = document.getElementById(POST_DATA);
const body = document.querySelector<HTMLElement>(".post-body");
if (data?.textContent && body) void live(body, JSON.parse(data.textContent) as { path: string; files: Record<string, string> });
