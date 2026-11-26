<!-- The preview (docs/editor-support.md, decision 8): the document's page full window, as the writer's editor has it, and
     the CompMD Inspector behind a tab, on the view the page renders. It has no text of its own: the server's events
     bring each change, which the Developer Tools' worker processes and the page splices, as the editor's do. -->
<script lang="ts">
  import type { Request } from "@compmd/processor";
  import { createSandbox, type SandboxOptions, type View } from "@compmd/sandbox";
  import { untrack } from "svelte";
  import type { DevtoolsDocument } from "./document.ts";
  import { Editor } from "./editor.ts";
  import { Follower, type WriterChange } from "./follow.ts";
  import Inspector from "./Inspector.svelte";
  import type { Registries, RegistryConfig } from "./protocol.ts";

  interface Props {
    sandbox: SandboxOptions;
    events: string;
    request: () => Promise<Request>;
    registries?: Registries;
    stylesheets?: readonly string[];
  }

  let { sandbox: sandboxOptions, events, request, registries, stylesheets = [] }: Props = $props();

  let tab: "page" | "inspector" = $state("page");
  /** Why the document couldn't be had from the server, until the next change. */
  let failure: string | undefined = $state();
  /** The page, and the document as it is now, which the Inspector processes again for its views. */
  let page: View | undefined = $state.raw();
  let inspected: DevtoolsDocument | undefined = $state.raw();
  let inspector: { open(document: DevtoolsDocument): void } | undefined = $state();

  /** Opens the page in `container`, follows the server's events, and gives what closes them. */
  function start(container: HTMLElement): () => void {
    const fetchedModules = new Map<string, Blob>();
    const sandbox = createSandbox({
      ...sandboxOptions,
      modules: (place) => {
        const code = fetchedModules.get(`${place.pack}@${place.version}/${place.path}`);
        return code ? { code } : sandboxOptions.modules?.(place);
      },
    });
    const worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
    if (registries) worker.postMessage({ type: "registry", ...registries } satisfies RegistryConfig);
    const view = sandbox.open(container, { components: [] });
    view.styles(stylesheets);
    // Only to another site, a reader's click the sandbox passes on: never a script or a page of the host's own.
    view.on("link", ({ url }) => {
      if (/^(https?|mailto):/i.test(url)) window.open(url, "_blank", "noopener,noreferrer");
    });
    const editor: Editor = new Editor(
      worker,
      view,
      () => {
        const document = editor.document;
        if (!document) return;
        inspected = { request: document };
        inspector?.open(inspected);
      },
      fetchedModules,
    );
    const follower = new Follower(editor, view, request, (message) => (failure = message));
    const source = new EventSource(events);
    source.addEventListener("change", ({ data }: MessageEvent<string>) => {
      failure = undefined;
      follower.change(JSON.parse(data) as WriterChange);
    });
    source.addEventListener("reload", () => {
      follower.reload();
    });
    page = view;
    return () => {
      source.close();
      view.destroy();
      worker.terminate();
    };
  }
</script>

<div class="devtools preview">
  <div class="tab-bar">
    <div class="tabs" role="tablist" aria-label="The preview">
      <button type="button" role="tab" aria-selected={tab === "page"} onclick={() => (tab = "page")}>Page</button>
      <button type="button" role="tab" aria-selected={tab === "inspector"} onclick={() => (tab = "inspector")}>CompMD Inspector</button>
    </div>
  </div>
  {#if failure}
    <p class="preview-failure" role="alert">{failure}</p>
  {/if}
  <div class="preview-page" hidden={tab !== "page"}>
    <div class="page-frame" {@attach (container) => untrack(() => start(container))}></div>
  </div>
  {#if page && inspected}
    <div class="preview-inspector" hidden={tab !== "inspector"}>
      <Inspector bind:this={inspector} view={page} document={inspected} />
    </div>
  {/if}
</div>
