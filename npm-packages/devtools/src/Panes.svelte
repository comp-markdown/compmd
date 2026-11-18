<!-- The tools' tabs and what each shows, for the editor and the inspector alike. With a page (the editor's), two tabs:
     the page, and the CompMD Inspector; the inspector alone shows the Inspector's own. The Inspector's tabs: what a
     plain Markdown renderer shows, the trees, the data, the components, the sandbox's frames and the messages; under
     them, the Console, in a drawer that hides and resizes. The views' worker makes the document's views, from the document as it is now, while their tab is open,
     and evaluates the Console's expressions in it; the observer's tabs read what the sandbox's observer kept. -->
<script lang="ts">
  import type { Diagnostic, ExpandedInput, Range, Request } from "@compmd/processor";
  import type { ObservedEvent } from "@compmd/sandbox";
  import { untrack, type Snippet } from "svelte";
  import ComponentsView from "./ComponentsView.svelte";
  import { now, type ConsoleEntry, type Evaluation } from "./console.ts";
  import { DRAWER_HEIGHT, DRAWER_STEP, drawerHeight } from "./drawer.ts";
  import ConsoleView from "./ConsoleView.svelte";
  import DataView from "./DataView.svelte";
  import { treeOf } from "./frames.ts";
  import Inputs from "./Inputs.svelte";
  import JsonTree from "./JsonTree.svelte";
  import { timelineOf } from "./messages.ts";
  import MessagesView from "./MessagesView.svelte";
  import SandboxView from "./SandboxView.svelte";
  import { isViewTab, kindOf, TABS, type Section, type Tab } from "./tabs.ts";
  import { plainPage, ViewsClient, type View } from "./views.ts";

  interface Props {
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
    /** The page tab's content; given, the tabs are the page and the CompMD Inspector. */
    page?: Snippet;
    /** Beside the tabs: how fast the last change reached the page. */
    status?: Snippet;
    /** The document's text as written, for Plain Text. */
    text?: string | undefined;
    /** The document's inputs, for the drawer's Inputs, with what sets one; without `onset`, there is no Inputs. */
    inputs?: Readonly<Record<string, ExpandedInput>> | undefined;
    onset?: ((name: string, value: unknown) => void) | undefined;
  }

  let { document, revision, observed, dropped, entries, onselect, ondiagnostics, page, status, text, inputs, onset }: Props = $props();

  /** The drawer's tab: the Console, or the document's inputs when the tools can set any. */
  let drawerTab = $state<"console" | "inputs">("console");
  const settable = $derived(onset !== undefined && inputs !== undefined && Object.keys(inputs).length > 0);
  const shownDrawer = $derived(settable ? drawerTab : "console");
  /** Shows the drawer, open, at `chosen`. */
  function openDrawer(chosen: "console" | "inputs"): void {
    drawerTab = chosen;
    consoleOpen = true;
  }

  /** The editor's tab, the page first; the inspector alone has the Inspector only. */
  let section: Section = $state(untrack(() => (page ? "page" : "inspector")));
  /** The Inspector's tab. */
  let tab: Tab = $state(TABS[0].id);
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

  /** The view the Inspector's tab shows, of the document as it is now, while the Inspector shows. */
  $effect(() => {
    void revision;
    if (document && section === "inspector" && isViewTab(tab)) client().request(asked(document), kindOf(tab));
  });

  const tree = $derived(treeOf(observed));
  const timeline = $derived(timelineOf(observed));

  /** The Console's drawer, under the Inspector's views: whether it shows, and how tall it is. */
  let consoleOpen = $state(true);
  let drawer = $state(DRAWER_HEIGHT);
  let pane: HTMLElement | undefined = $state();
  // A pane not laid out yet has no height: nothing bounds the drawer then.
  const setDrawer = (wanted: number) => {
    drawer = drawerHeight(wanted, pane?.clientHeight || Number.POSITIVE_INFINITY);
  };

  /** Drags the drawer's top edge: the pointer stays the handle's, over the page's frames too. */
  function resize(event: PointerEvent & { currentTarget: HTMLElement }): void {
    const handle = event.currentTarget;
    handle.setPointerCapture(event.pointerId);
    const [from, height] = [event.clientY, drawer];
    const move = (moved: PointerEvent) => {
      setDrawer(height - (moved.clientY - from));
    };
    const end = () => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", end);
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", end);
  }

  function resizeByKey(event: KeyboardEvent): void {
    const step = event.key === "ArrowUp" ? DRAWER_STEP : event.key === "ArrowDown" ? -DRAWER_STEP : 0;
    if (step === 0) return;
    event.preventDefault();
    setDrawer(drawer + step);
  }

  function toggleOnEscape(event: KeyboardEvent): void {
    if (event.key !== "Escape" || event.defaultPrevented) return;
    event.preventDefault();
    consoleOpen = !consoleOpen;
  }
</script>

<div class="sheet views">
  {#if page}
    <div class="tab-bar">
      <div class="tabs" role="tablist" aria-label="The page and the CompMD Inspector">
        <button type="button" role="tab" id="tab-page" aria-selected={section === "page"} aria-controls="view-page" onclick={() => (section = "page")}>Page</button>
        <button type="button" role="tab" id="tab-inspector" aria-selected={section === "inspector"} aria-controls="view-inspector" onclick={() => (section = "inspector")}>CompMD Inspector</button>
      </div>
      {@render status?.()}
    </div>
    <!-- The page stays mounted under the Inspector, so its components keep their state. -->
    <div class="view page-view" id="view-page" role="tabpanel" aria-labelledby="tab-page" hidden={section !== "page"}>
      {@render page()}
    </div>
  {/if}
  {#if section === "inspector"}
    <!-- Esc shows or hides the Console, as in a browser's developer tools. -->
    <div class="inspector-pane" id="view-inspector" role={page ? "tabpanel" : undefined} aria-labelledby={page ? "tab-inspector" : undefined} bind:this={pane} onkeydown={toggleOnEscape}>
      <div class="tab-bar inspector-tabs">
        <div class="tabs" role="tablist" aria-label="The CompMD Inspector's views">
          {#each TABS as { id, label } (id)}
            <button type="button" role="tab" id="tab-{id}" aria-selected={tab === id} aria-controls="view-{id}" onclick={() => (tab = id)}>{label}</button>
          {/each}
        </div>
        {#if !page}{@render status?.()}{/if}
      </div>
      <div class="view" id="view-{tab}" role="tabpanel" aria-labelledby="tab-{tab}">
        {#if tab === "sandbox"}
          <SandboxView {tree} {dropped} />
        {:else if tab === "messages"}
          <MessagesView {timeline} />
        {:else if tab === "text"}
          <pre class="plain-text" aria-label="The document's text, as written">{text ?? ""}</pre>
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
      <section class="drawer" aria-label="The drawer">
        <div class="drawer-bar">
          <!-- A window splitter, as the text's and the views' is. -->
          <!-- svelte-ignore a11y_no_noninteractive_tabindex, a11y_no_noninteractive_element_interactions -->
          <div
            class="drawer-handle"
            role="separator"
            aria-orientation="horizontal"
            aria-label="The drawer's height: drag, or the arrow keys"
            aria-valuenow={drawer}
            tabindex="0"
            hidden={!consoleOpen}
            onpointerdown={resize}
            onkeydown={resizeByKey}
          ></div>
          <button
            type="button"
            class="drawer-toggle"
            aria-expanded={consoleOpen}
            aria-controls="drawer-body"
            aria-label="Show or hide the drawer"
            title="Show or hide the drawer (Esc)"
            onclick={() => (consoleOpen = !consoleOpen)}
          >
            <svg viewBox="0 0 10 10" aria-hidden="true"><path d="m3 2 4 3-4 3" /></svg>
          </button>
          <div class="tabs drawer-tabs" role="tablist" aria-label="The drawer">
            <button type="button" role="tab" id="drawer-console" aria-selected={shownDrawer === "console"} aria-controls="drawer-body" onclick={() => openDrawer("console")}>Console</button>
            {#if settable}
              <button type="button" role="tab" id="drawer-inputs" aria-selected={shownDrawer === "inputs"} aria-controls="drawer-body" onclick={() => openDrawer("inputs")}>Inputs</button>
            {/if}
          </div>
        </div>
        {#if consoleOpen}
          <div class="drawer-body" id="drawer-body" role="tabpanel" aria-labelledby="drawer-{shownDrawer}" style:height="{drawer}px">
            {#if shownDrawer === "inputs" && inputs && onset}
              <Inputs {inputs} {onset} />
            {:else}
              <ConsoleView {entries} {evaluations} onevaluate={document ? evaluate : undefined} {onselect} />
            {/if}
          </div>
        {/if}
      </section>
    </div>
  {/if}
</div>
