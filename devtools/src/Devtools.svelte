<!-- The Developer Tools' editor (docs/devtools.md): the document's text on the left, with the hint at the caret and the
     diagnostics under it; on the right, the document's inputs over tabs with the rendered page in its sandbox, what went
     wrong, the components the document can name, the sandbox's frames, what the frontmatter gives the render, what a
     plain Markdown renderer shows, the messages between the frames, and the trees. Each document opens with a worker
     and a page of its own; the sandbox's observer watches the page from its start. -->
<script lang="ts">
  import type { Range } from "@compmd/processor";
  import { createSandbox, type Observation, type ObservedEvent, type SandboxOptions, type View as Page } from "@compmd/sandbox";
  import { tick, untrack } from "svelte";
  import { caretTop } from "./caret.ts";
  import ComponentsView from "./ComponentsView.svelte";
  import { consoleOf, EditorHistory } from "./console.ts";
  import ConsoleView from "./ConsoleView.svelte";
  import DataView from "./DataView.svelte";
  import type { DevtoolsDocument, SupportingFile } from "./document.ts";
  import { offsetOf } from "./edit.ts";
  import { Editor, type EditorState, type Replaced } from "./editor.ts";
  import { directiveFor, findPassage, fragmentOf, parseFragment, type Passage } from "./fragment.ts";
  import { treeOf } from "./frames.ts";
  import { lineOf } from "./hint.ts";
  import Inputs from "./Inputs.svelte";
  import JsonTree from "./JsonTree.svelte";
  import { timelineOf } from "./messages.ts";
  import MessagesView from "./MessagesView.svelte";
  import type { Registries, RegistryConfig } from "./protocol.ts";
  import SandboxView from "./SandboxView.svelte";
  import Split from "./Split.svelte";
  import { plainPage, ViewsClient, type View, type ViewKind } from "./views.ts";

  interface Props {
    sandbox: SandboxOptions;
    document: DevtoolsDocument;
    registries?: Registries;
    stylesheets?: readonly string[];
    select?: string;
    shareLink?: (select: string) => string;
  }

  let { sandbox: sandboxOptions, document: first, registries, stylesheets: firstStylesheets = [], select: firstSelect, shareLink }: Props = $props();

  /** The document open, and how many have been: a new one opens a new page. */
  let opened = $state.raw(untrack(() => first));
  let generation = $state(0);
  let text = $state("");
  let shown: EditorState | undefined = $state.raw();
  let textarea: HTMLTextAreaElement | undefined = $state();
  /** The supporting file shown in place of the text, read-only; `undefined` for the document. */
  let file: SupportingFile | undefined = $state.raw();
  let editor: Editor | undefined;
  let worker: Worker | undefined;

  const TABS = [
    { id: "page", label: "Page" },
    { id: "console", label: "Console" },
    { id: "components", label: "Components" },
    { id: "sandbox", label: "Sandbox" },
    { id: "data", label: "Data" },
    { id: "plain", label: "Plain Markdown" },
    { id: "messages", label: "Messages" },
    { id: "parse", label: "Parse tree" },
    { id: "expanded", label: "Expanded tree" },
  ] as const;
  type Tab = (typeof TABS)[number]["id"];
  /** The tabs the observer fills, which the views' worker doesn't. */
  type Observed = "console" | "sandbox" | "messages";
  let tab: Tab = $state("page");
  let view: View | undefined = $state.raw();
  let views: ViewsClient | undefined;
  /** How deep the trees open: two levels at first; "Open all" and "Fold all" set it. */
  let treeDepth = $state(2);

  const isObserved = (shown: Tab): shown is Observed => shown === "console" || shown === "sandbox" || shown === "messages";
  /** The view the views' worker makes for `tab`. */
  const kindOf = (shown: Exclude<Tab, "page" | Observed>): ViewKind => (shown === "parse" || shown === "expanded" ? "trees" : shown);

  /** The view the tab shows, of the document as it is now; the views' worker starts the first time one is asked for. */
  $effect(() => {
    if (tab === "page" || isObserved(tab) || shown === undefined) return;
    void shown.updates;
    const document = editor?.document;
    if (!document) return;
    views ??= new ViewsClient(new Worker(new URL("./views-worker.js", import.meta.url), { type: "module" }), (answer) => {
      view = answer;
    });
    // Without the origins the editor's keys need, which would make the expanded tree three times as long.
    views.request({ ...document, origins: false }, kindOf(tab));
  });

  /** What the sandbox's observer kept of the page, read again once a frame after it changes. */
  let observation: Observation | undefined;
  let observed: readonly ObservedEvent[] = $state.raw([]);
  let dropped: Readonly<Record<string, number>> = $state.raw({});
  let reading = false;
  function readLater(): void {
    if (reading) return;
    reading = true;
    requestAnimationFrame(() => {
      reading = false;
      observed = observation?.events ?? [];
      dropped = observation?.dropped ?? {};
    });
  }
  /** What the editor said went wrong, for the console; `noted` counts its changes. */
  let history = new EditorHistory();
  let noted = $state(0);
  const tree = $derived(treeOf(observed));
  const timeline = $derived(timelineOf(observed));
  const entries = $derived.by(() => {
    void noted;
    return consoleOf(history.entries, observed);
  });

  let stylesheets = $state.raw(untrack(() => firstStylesheets));
  let page: Page | undefined;
  $effect(() => {
    page?.styles(stylesheets);
  });

  /** The registries, for the worker, which fetches the packs a document imports from them. */
  const registryConfig: RegistryConfig | undefined = untrack(() => registries && { type: "registry", ...registries });

  /** The built modules the worker fetched from the registry, checked, by `pack@version/path`: the sandbox loads them as code. */
  const fetchedModules = new Map<string, Blob>();
  /** The tools' sandbox: the host's options, with the built modules fetched from the registry given as code. */
  const sandbox = untrack(() =>
    createSandbox({
      ...sandboxOptions,
      modules: (place) => {
        const code = fetchedModules.get(`${place.pack}@${place.version}/${place.path}`);
        return code ? { code } : sandboxOptions.modules?.(place);
      },
    }),
  );

  /** Opens `document` in place of the one open, in a new page. */
  export function open(document: DevtoolsDocument): void {
    file = undefined;
    opened = document;
    generation += 1;
  }

  /** The host's stylesheets for the page, in place of those it had. */
  export function styles(urls: readonly string[]): void {
    stylesheets = urls;
  }

  /** The editor's state: its diagnostics, problems and timings. */
  export function editorState(): EditorState | undefined {
    return editor?.state;
  }

  /** Selects `range` in the text area (to the end of its line, when it has no end), and scrolls to it. */
  async function select(range: Range): Promise<void> {
    file = undefined;
    await tick();
    if (!textarea) return;
    const start = offsetOf(textarea.value, range.start);
    const lineEnd = textarea.value.indexOf("\n", start);
    const end = range.end ? offsetOf(textarea.value, range.end) : lineEnd === -1 ? textarea.value.length : lineEnd;
    selectText({ start, end });
  }

  /** Selects `passage` of the text area, and scrolls it to near the top, a few lines below it. */
  function selectText({ start, end }: Passage): void {
    if (!textarea) return;
    textarea.focus();
    textarea.setSelectionRange(start, end);
    const lineHeight = Number.parseFloat(getComputedStyle(textarea).lineHeight) || 20;
    textarea.scrollTop = Math.max(0, caretTop(textarea, start) - 2 * lineHeight);
    noteSelection();
  }

  /** Why a link to a passage of the text found none; cleared by the next input. */
  let linkNote: string | undefined = $state();

  /** Selects the passage of the text a text-fragment `link` names (`#:~:text=…`), or says it isn't there. */
  async function showPassage(link: string): Promise<void> {
    const directive = parseFragment(link);
    if (!directive) return;
    file = undefined;
    await tick();
    const passage = textarea && findPassage(textarea.value, directive);
    if (!passage) {
      linkNote = `The linked text, “${directive.start}”, isn't in the document now.`;
      return;
    }
    linkNote = undefined;
    selectText(passage);
  }

  /** The passage a shared link names, as written: its parts are percent-encoded as a text fragment's. */
  let shared = untrack(() => firstSelect);

  /** Whether text is selected, which offers a link to it. */
  let selected = $state(false);
  let copied = $state(false);
  /** Notes what is selected, and where the caret is, for the hint at it: the session asks the worker once it is free. */
  function noteSelection(): void {
    selected = textarea !== undefined && textarea.selectionStart !== textarea.selectionEnd;
    if (textarea) editor?.moveCaret(textarea.selectionStart);
  }

  /** Puts the text a fix or a value button wrote in the text area, with the caret past what it wrote. */
  function write(replaced: Replaced | undefined): void {
    if (!textarea || !replaced) return;
    textarea.value = replaced.text;
    textarea.focus();
    textarea.setSelectionRange(replaced.caret, replaced.caret);
  }

  /** Copies a link the host makes, to this document with the selected passage, or shows it when it can't. */
  async function copyLink(): Promise<void> {
    if (!textarea || !shareLink) return;
    const directive = directiveFor(textarea.value, { start: textarea.selectionStart, end: textarea.selectionEnd });
    if (!directive) return;
    const url = shareLink(fragmentOf(directive).slice(":~:text=".length));
    try {
      await navigator.clipboard.writeText(url);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      prompt("Copy this link:", url);
    }
  }

  /** Opens the document in a page of its own, in `container`; the returned function closes it. */
  function start(container: HTMLElement): () => void {
    worker?.terminate();
    worker = new Worker(new URL("./worker.js", import.meta.url), { type: "module" });
    if (registryConfig) worker.postMessage(registryConfig);
    const view = sandbox.open(container, { components: [] });
    observation?.stop();
    observation = view.observe(readLater);
    history = new EditorHistory();
    view.styles(stylesheets);
    view.on("link", ({ url }) => {
      if (url.includes(":~:text=")) void showPassage(url);
      // Only to another site, a reader's click the sandbox passes on: never a script or a page of the host's own.
      else if (/^(https?|mailto):/i.test(url)) window.open(url, "_blank", "noopener,noreferrer");
    });
    page = view;
    const editing = new Editor(
      worker,
      view,
      () => {
        shown = { ...editing.state };
        history.note(editing.state.diagnostics, editing.state.problems);
        noted += 1;
      },
      fetchedModules,
    );
    editor = editing;
    text = editing.open(opened);
    // A shared link selects its passage once, in the document it opened with.
    if (shared !== undefined) void showPassage(`:~:text=${shared}`);
    shared = undefined;
    return () => {
      observation?.stop();
      view.destroy();
    };
  }

  const ms = (value: number | undefined) => (value === undefined ? "–" : `${value.toFixed(1)} ms`);
</script>

<div class="devtools">
  <Split text={source} views={panes} />
</div>

{#snippet source()}
  <div class="sheet text">
    {#if opened.files && opened.files.length > 0}
      <div class="tabs files" role="tablist" aria-label="The document and the files it reads">
        <button type="button" role="tab" aria-selected={file === undefined} onclick={() => (file = undefined)}>{opened.request.path}</button>
        {#each opened.files as supporting (supporting.name)}
          <button type="button" role="tab" aria-selected={file === supporting} title="Read-only: a file the document reads" onclick={() => (file = supporting)}>
            {supporting.name}
          </button>
        {/each}
      </div>
    {/if}
    <div class="text-area">
      {#if file}
        <pre class="file" aria-label="{file.name}, read-only">{file.text}</pre>
      {/if}
      <textarea
        hidden={file !== undefined}
        bind:this={textarea}
        value={text}
        spellcheck="false"
        aria-label="The document's text"
        onbeforeinput={() => {
          if (textarea) editor?.beforeInput({ start: textarea.selectionStart, end: textarea.selectionEnd });
        }}
        oninput={(event) => {
          if (textarea) editor?.input(textarea.value, textarea.selectionStart, event.timeStamp);
          linkNote = undefined;
          noteSelection();
        }}
        onselect={noteSelection}
        onselectionchange={noteSelection}
        onkeyup={noteSelection}
        onmouseup={noteSelection}
      ></textarea>
      {#if selected && file === undefined && shareLink}
        <button type="button" class="copy-link" title="Copy a link that opens this document with this text selected" onclick={() => void copyLink()}>
          {copied ? "Copied" : "Copy link to selection"}
        </button>
      {/if}
    </div>
    {#if linkNote}
      <p class="link-note" role="status">{linkNote}</p>
    {/if}
    {#if shown?.hint && file === undefined}
      {@const hint = shown.hint}
      {@const line = lineOf(hint)}
      <div class="hint" role="status" aria-label="What is at the caret">
        {#each hint.diagnostics as d, i (i)}
          <span class="part diagnostic {d.severity}">
            <code class="code">{d.code}</code>{d.message}
            {#if d.fix}
              {@const fix = d.fix}
              <button type="button" class="fix" onmousedown={(event) => event.preventDefault()} onclick={() => write(editor?.fix(fix))}>Use <code>{fix.text}</code></button>
            {/if}
          </span>
        {/each}
        {#if line}
          {#each line.parts as part, i (i)}
            <span class="part"
              >{#each part as run, j (j)}{#if run.name}<code class="name">{run.text}</code>{:else if run.code}<code>{run.text}</code>{:else}{run.text}{/if}{/each}</span
            >
            {#if i === 0 && line.values.length > 0}
              <span class="part values">
                {#each line.values as value (value.text)}
                  <button
                    type="button"
                    class="value"
                    aria-pressed={value.text === line.current}
                    title={value.detail}
                    onmousedown={(event) => event.preventDefault()}
                    onclick={() => write(editor?.complete(value.text))}><code>{value.label}</code></button
                  >
                {/each}
              </span>
            {/if}
          {/each}
        {/if}
      </div>
    {/if}
    {#if shown && (shown.diagnostics.length > 0 || shown.problems.length > 0)}
      <ul class="diagnostics" aria-label="Diagnostics">
        {#each shown.diagnostics as d, i (i)}
          <li class={d.severity}>
            <code class="code">{d.code}</code>
            {#if d.file === undefined}
              <button type="button" class="place" title="Select it in the text" onclick={() => void select(d.range)}>{d.range.start.line}:{d.range.start.column}</button>
            {:else}
              {d.file} {d.range.start.line}:{d.range.start.column}
            {/if}
            {d.message}
            {#if d.fix && d.file === undefined}
              {@const fix = d.fix}
              <button type="button" class="fix" onclick={() => write(editor?.fix(fix))}>Use <code>{fix.text}</code></button>
            {/if}
          </li>
        {/each}
        {#each shown.problems as problem (problem)}
          <li class="error problem">{problem}</li>
        {/each}
      </ul>
    {/if}
  </div>
{/snippet}

{#snippet panes()}
  <div class="sheet views">
    {#if shown && Object.keys(shown.inputs).length > 0}
      <Inputs inputs={shown.inputs} onset={(name, value) => editor?.setInput(name, value)} />
    {/if}
    <div class="tab-bar">
      <div class="tabs" role="tablist" aria-label="Views">
        {#each TABS as { id, label } (id)}
          <button type="button" role="tab" id="tab-{id}" aria-selected={tab === id} aria-controls="view-{id}" onclick={() => (tab = id)}>{label}</button>
        {/each}
      </div>
      <dl class="status" aria-live="off">
        <div title="The worker's processing of the last change; p95 {ms(shown?.processing.p95)}">
          <dt>Processing</dt>
          <dd>{ms(shown?.processing.last)}</dd>
        </div>
        <div title="From the last change's first key press to the page painted; p95 {ms(shown?.painted.p95)}">
          <dt>Key press to paint</dt>
          <dd>{ms(shown?.painted.last)}</dd>
        </div>
      </dl>
    </div>
    <!-- The page stays mounted under the other tabs, so its components keep their state. -->
    <div class="view page-view" id="view-page" role="tabpanel" aria-labelledby="tab-page" hidden={tab !== "page"}>
      {#key generation}
        <div class="page-frame" {@attach (container) => untrack(() => start(container))}></div>
      {/key}
    </div>
    {#if tab !== "page"}
      <div class="view" id="view-{tab}" role="tabpanel" aria-labelledby="tab-{tab}">
        {#if tab === "console"}
          <ConsoleView {entries} onselect={(range) => void select(range)} />
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
          <ComponentsView components={view.components} onselect={(range) => void select(range)} />
        {:else}
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
{/snippet}
