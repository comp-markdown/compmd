// A document of the page rendered in a sandbox, as a host renders it: in frame mode, the whole page in a strict frame,
// which runs its JSX, Svelte and JavaScript components; in host mode, in this page, the site's own trusted components
// inline (its `stars`) and every other component in a frame. The frame documents and the site's packs are the site's
// own, from `@compmd/vite`; the page processes the document and gives the sandbox its tree and its components.
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";
import { request, sandbox } from "virtual:compmd";

/** The playground's host stylesheets, from this page. */
const PLAYGROUND = new URL("playground/", location.href).href;

/**
 * Renders the document `path`, whose text is `text`, in `container`, in a frame (`frame`) or in this page (`host`);
 * `files` are the other files it reads. `onShown` is called once the page shows with its components, which load after
 * it renders: until then its places would show empty. Gives what closes it.
 */
export async function renderInFrame(container: HTMLElement, path: string, text: string, files: Record<string, string>, onShown: () => void, page: "frame" | "host" = "frame"): Promise<() => void> {
  await init();
  let inputs: Record<string, unknown> = {};
  const open = () => new OpenDocument({ ...request, path, files: { ...files, [path]: text }, now: new Date().toISOString(), inputs });
  let document = open();
  const view = createSandbox({ ...sandbox, page, stylesheets: [`${PLAYGROUND}frame.css?host=reference`] }).open(container, {
    components: document.components,
  });
  let rendered = false;
  let loaded = false;
  const showIfReady = () => {
    if (rendered && loaded) onShown();
  };
  view.on("loaded", () => {
    loaded = true;
    showIfReady();
  });
  view.on("rendered", () => {
    rendered = true;
    showIfReady();
  });
  view.on("set", ({ input, value }) => {
    inputs = { ...inputs, [input]: value };
    document.free();
    document = open();
    view.render(document.expanded);
  });
  view.render(document.expanded);
  return () => {
    view.destroy();
    document.free();
  };
}
