// A document of the page rendered in the playground's frame, the sandbox that runs its JSX, Svelte and JavaScript
// components: a sandboxed iframe with an opaque origin, loaded from the playground beside this page. The page
// processes the document and sends the frame its tree and its components' code, as the playground does.
import { init, OpenDocument } from "@compmd/processor";
import { connectFrame } from "@compmd/render/frame";
import { FRAME_HELLO, type ComponentsLoaded, type ComponentsMessage, type HostStyleMessage } from "../../editor/src/protocol.ts";

/**
 * Renders the document `path`, whose text is `text`, in `iframe` once it has loaded the playground's frame; `files`
 * are the other files it reads. `onShown` is called once the frame shows the page with its components, which load
 * after the page renders: until then the frame would show their places empty. Gives what closes it.
 */
export async function renderInFrame(iframe: HTMLIFrameElement, path: string, text: string, files: Record<string, string>, onShown: () => void): Promise<() => void> {
  await init();
  let inputs: Record<string, unknown> = {};
  const open = () => new OpenDocument({ path, files: { ...files, [path]: text }, now: new Date().toISOString(), inputs });
  let document = open();
  const { port1, port2 } = new MessageChannel();
  let rendered = false;
  let loaded = false;
  const showIfReady = () => {
    if (rendered && loaded) onShown();
  };
  port1.addEventListener("message", ({ data }: MessageEvent<Partial<ComponentsLoaded> | { type?: string }>) => {
    if (data.type !== "components-loaded") return;
    loaded = true;
    showIfReady();
  });
  iframe.contentWindow?.postMessage(FRAME_HELLO, "*", [port2]);
  // Before the tree, as the frame compiles the components before it renders.
  port1.postMessage({ type: "components", components: document.components } satisfies ComponentsMessage);
  port1.postMessage({ type: "host-style", style: "reference" } satisfies HostStyleMessage);
  const frame = connectFrame(port1);
  frame.on("rendered", () => {
    rendered = true;
    showIfReady();
  });
  frame.on("set", ({ input, value }) => {
    inputs = { ...inputs, [input]: value };
    document.free();
    document = open();
    frame.render(document.expanded);
  });
  frame.render(document.expanded);
  return () => {
    frame.close();
    port1.close();
    document.free();
  };
}
