<!-- The tools' tabs and what each shows, for the editor and the inspector alike: the page (the editor's), what went
     wrong, the components, the sandbox's frames, the data, what a plain Markdown renderer shows, the messages and the
     trees. The views' worker makes the document's views, from the document as it is now, while their tab is open, and
     evaluates the Console's expressions in it; the observer's tabs read what the sandbox's observer kept. -->
<script lang="ts">
  import type { Diagnostic, Range, Request } from "@compmd/processor";
  import type { ObservedEvent } from "@compmd/sandbox";
  import { untrack, type Snippet } from "svelte";
  import ComponentsView from "./ComponentsView.svelte";
  import { now, type ConsoleEntry, type Evaluation } from "./console.ts";
  import ConsoleView from "./ConsoleView.svelte";
  import DataView from "./DataView.svelte";
  import { treeOf } from "./frames.ts";
  import JsonTree from "./JsonTree.svelte";
  import { timelineOf } from "./messages.ts";
  import MessagesView from "./MessagesView.svelte";
  import SandboxView from "./SandboxView.svelte";
  import { isViewTab, kindOf, TABS, type Tab } from "./tabs.ts";
  import { plainPage, ViewsClient, type View } from "./views.ts";

  interface Props {
    /** The tabs, in order. */
    tabs: readonly Tab[];
    /** The document as it is now, and a number that changes with it. */
    document: Request | undefined;
    revision: number;
    observed: readonly ObservedEvent[];
    dropped: Readonly<Record<string, number>>;
    entries: readonly ConsoleEntry[];
    /** Selects a place in the text; without it, places are text. */
    onselect?: (range: Range) => void;
    /** Given, the diagnostics are asked for at each revision, and given to it. */
    ondiagnostics?: (diagnostics: Diagnostic[]) => void;
    /** The page tab's content. */
    page?: Snippet;
    /** Over the tabs: the document's inputs. */
    above?: Snippet;
    /** Beside the tabs: how fast the last change reached the page. */
    status?: Snippet;
  }

  let { tabs, document, revision, observed, dropped, entries, onselect, ondiagnostics, page, above, status }: Props = $props();

  let tab: Tab = $state(untrack(() => tabs[0]) ?? "console");
  let view: View | undefined = $state.raw();
  /** How deep the trees open: two levels at first; "Open all" and "Fold all" set it. */
  let treeDepth = $state(2);

  /** The expressions the Console evaluated in the document, each with its result once the views' worker gives it. */
  let evaluations: Evaluation[] = $state.raw([]);

  let views: ViewsClient | undefined;
  /** The views' worker's client, which starts the worker the first time a view is asked for. */
  const client = (): ViewsClient =>
    (views ??= new ViewsClient(new Worker(new URL("./views-worker.js", import.meta.url), { type: "module" }), (answer) => {
      if (answer.view === "evaluate") {
        const result = "failed" in answer ? { failed: answer.failed } : answer.result;
        evaluations = evaluations.map((evaluation) => (evaluation.id === answer.id ? { ...evaluation, result } : evaluation));
      } else if (answer.view !== "diagnostics") view = answer;
      else if ("diagnostics" in answer) ondiagnostics?.(answer.diagnostics);
    }));

  // Without the origins the editor's keys need, which would make the expanded tree three times as long.
  const asked = (request: Request): Request => ({ ...request, origins: false });

  /** Evaluates `expression` in the document as it is now, in the views' worker. */
  function evaluate(expression: string): void {
    if (!document) return;
    const id = (evaluations.at(-1)?.id ?? 0) + 1;
    evaluations = [...evaluations, { id, at: now(), expression }];
    client().evaluate(asked(document), id, expression);
  }

  $effect(() => {
    void revision;
    if (document && ondiagnostics) client().request(asked(document), "diagnostics");
  });

  /** The view the tab shows, of the document as it is now. */
  $effect(() => {
    void revision;
    if (document && isViewTab(tab)) client().request(asked(document), kindOf(tab));
  });

  const tree = $derived(treeOf(observed));
  const timeline = $derived(timelineOf(observed));
  const shown = $derived(TABS.filter(({ id }) => tabs.includes(id)));
</script>

<div class="sheet views">
  {@render above?.()}
  <div class="tab-bar">
    <div class="tabs" role="tablist" aria-label="Views">
      {#each shown as { id, label } (id)}
        <button type="button" role="tab" id="tab-{id}" aria-selected={tab === id} aria-controls="view-{id}" onclick={() => (tab = id)}>{label}</button>
      {/each}
    </div>
    {@render status?.()}
  </div>
  {#if page}
    <!-- The page stays mounted under the other tabs, so its components keep their state. -->
    <div class="view page-view" id="view-page" role="tabpanel" aria-labelledby="tab-page" hidden={tab !== "page"}>
      {@render page()}
    </div>
  {/if}
  {#if tab !== "page"}
    <div class="view" id="view-{tab}" role="tabpanel" aria-labelledby="tab-{tab}">
      {#if tab === "console"}
        <ConsoleView {entries} {evaluations} onevaluate={document ? evaluate : undefined} {onselect} />
      {:else if tab === "sandbox"}
        <SandboxView {tree} {dropped} />
      {:else if tab === "messages"}
        <MessagesView {timeline} />
      {:else if view === undefined || view.view !== kindOf(tab)}
        <p class="note">Processing…</p>
      {:else if "failed" in view}
        <p class="note error">{view.failed}</p>
      {:else if "html" in view}
        <!-- No scripts and an opaque origin: the HTML is markdown-rs's, with raw HTML escaped, but it is the document's. -->
        <iframe title="What a plain Markdown renderer shows" sandbox="" srcdoc={plainPage(view.html)}></iframe>
      {:else if "frontmatter" in view}
        <DataView frontmatter={view.frontmatter} />
      {:else if "components" in view}
        <ComponentsView components={view.components} {onselect} />
      {:else if "parse" in view}
        <div class="tree-tools">
          <button type="button" onclick={() => (treeDepth = Number.POSITIVE_INFINITY)}>Open all</button>
          <button type="button" onclick={() => (treeDepth = 1)}>Fold all</button>
        </div>
        <div class="tree">
          <JsonTree value={tab === "parse" ? view.parse : view.expanded} open={treeDepth} />
        </div>
      {/if}
    </div>
  {/if}
</div>
