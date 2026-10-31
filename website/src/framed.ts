// A document of the page rendered in a sandbox, as the playground renders its pages: `@compmd/sandbox`'s strict frame,
// which runs its JSX, Svelte and JavaScript components, from the playground's frame documents beside this page. The page
// processes the document and gives the sandbox its tree and its components.
import { init, OpenDocument } from "@compmd/processor";
import { createSandbox } from "@compmd/sandbox";

/** The playground's frame documents and host stylesheets, from this page. */
const PLAYGROUND = new URL("playground/", location.href).href;

/**
 * Renders the document `path`, whose text is `text`, in a frame the sandbox puts in `container`; `files` are the other
 * files it reads. `onShown` is called once the page shows with its components, which load after it renders: until then
 * its places would show empty. Gives what closes it.
 */
export async function renderInFrame(container: HTMLElement, path: string, text: string, files: Record<string, string>, onShown: () => void): Promise<() => void> {
  await init();
  let inputs: Record<string, unknown> = {};
  const open = () => new OpenDocument({ path, files: { ...files, [path]: text }, now: new Date().toISOString(), inputs });
  let document = open();
  const view = createSandbox({ base: `${PLAYGROUND}sandbox/`, stylesheets: [`${PLAYGROUND}frame.css?host=reference`] }).open(container, {
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
