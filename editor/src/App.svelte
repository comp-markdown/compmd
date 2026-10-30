<!-- The editor: the document's text on the left, with the hint at the caret and the diagnostics under it; on the right, the host's stylesheet
     the author picks and the document's inputs over tabs with the rendered page in a sandboxed frame, what a plain
     Markdown renderer shows, what the frontmatter gives the render, the components the document can name, and its
     trees. Each document opens with a worker and a frame of its own. -->
<script lang="ts">
  import type { Range } from "@compmd/processor";
  import { connectFrame } from "@compmd/render/frame";
  import { tick } from "svelte";
  import ComponentsView from "./ComponentsView.svelte";
  import DataView from "./DataView.svelte";
  import { EXAMPLES, exampleIndex, type SupportingFile } from "./documents.ts";
  import { offsetOf } from "./edit.ts";
  import { Editor, type EditorState, type Frame, type Replaced } from "./editor.ts";
  import { lineOf } from "./hint.ts";
  import Inputs from "./Inputs.svelte";
  import JsonTree from "./JsonTree.svelte";
  import Split from "./Split.svelte";
  import { caretTop } from "./caret.ts";
  import { directiveFor, findPassage, fragmentOf, parseFragment, type Passage } from "./fragment.ts";
  import { FRAME_HELLO, HOST_STYLES, type ComponentFailed, type HostStyle, type OpenLink, type SourceLink } from "./protocol.ts";
  import { keep, recall } from "./stored.ts";
  import { plainPage, ViewsClient, type View, type ViewKind } from "./views.ts";
  import markDark from "../../logo/mark-dark.svg";
  import markLight from "../../logo/mark-light.svg";

  let chosen = $state(exampleIndex(new URLSearchParams(location.search).get("doc")));
  let text = $state("");
  let shown: EditorState | undefined = $state.raw();
  let textarea: HTMLTextAreaElement | undefined = $state();
  const example = $derived(EXAMPLES[chosen]);
  /** The supporting file shown in place of the text, read-only; `undefined` for the document. */
  let file: SupportingFile | undefined = $state.raw();
  let editor: Editor | undefined;
  let worker: Worker | undefined;

  const TABS = [
    { id: "page", label: "CompMD" },
    { id: "plain", label: "Plain Markdown" },
    { id: "data", label: "Data" },
    { id: "components", label: "Components" },
    { id: "parse", label: "Parse tree" },
    { id: "expanded", label: "Expanded tree" },
  ] as const;
  type Tab = (typeof TABS)[number]["id"];
  let tab: Tab = $state("page");
  let view: View | undefined = $state.raw();
  let views: ViewsClient | undefined;
  /** How deep the trees open: two levels at first; "Open all" and "Fold all" set it. */
  let treeDepth = $state(2);

  /** The view the views' worker makes for `tab`. */
  const kindOf = (shown: Exclude<Tab, "page">): ViewKind => (shown === "parse" || shown === "expanded" ? "trees" : shown);

  /** The view the tab shows, of the document as it is now; the views' worker starts the first time one is asked for. */
  $effect(() => {
    if (tab === "page" || shown === undefined) return;
    void shown.updates;
    const document = editor?.document;
    if (!document) return;
    views ??= new ViewsClient(new Worker(new URL("./views-worker.ts", import.meta.url), { type: "module" }), (answer) => {
      view = answer;
    });
    // Without the origins the editor's keys need, which would make the expanded tree three times as long.
    views.request({ ...document, origins: false }, kindOf(tab));
  });

  /** The host's stylesheet the page shows in, remembered between visits. */
  const STYLE_KEY = "compmd-editor-host-style";
  let hostStyle: HostStyle = $state(HOST_STYLES.find(({ id }) => id === recall(STYLE_KEY))?.id ?? "reference");
  let frame: Frame | undefined;
  $effect(() => {
    keep(STYLE_KEY, hostStyle);
    frame?.send({ type: "host-style", style: hostStyle });
  });

  /** What the page is for: open on the first visit, until it is closed. */
  const ABOUT_KEY = "compmd-editor-about";
  let about = $state(recall(ABOUT_KEY) !== "closed");
  function toggleAbout(): void {
    about = !about;
    keep(ABOUT_KEY, about ? "open" : "closed");
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

  /** The passage a shared link names (`?select=`), as written: its parts are percent-encoded as a text fragment's. */
  let shared = /[?&]select=([^&]*)/.exec(location.search)?.[1];

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

  /** Copies a link to the playground with this document and the selected passage, or shows it when it can't. */
  async function copyLink(): Promise<void> {
    if (!textarea || !example) return;
    const directive = directiveFor(textarea.value, { start: textarea.selectionStart, end: textarea.selectionEnd });
    if (!directive) return;
    const url = `${location.origin}${location.pathname}?doc=${example.id}&select=${fragmentOf(directive).slice(":~:text=".length)}`;
    try {
      await navigator.clipboard.writeText(url);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      prompt("Copy this link:", url);
    }
  }

  /** Opens the chosen document once its frame has loaded. */
  function start(iframe: HTMLIFrameElement): void {
    worker?.terminate();
    worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    const { port1, port2 } = new MessageChannel();
    iframe.contentWindow?.postMessage(FRAME_HELLO, "*", [port2]);
    const connected: Frame = {
      connection: connectFrame(port1),
      send: (message) => {
        port1.postMessage(message);
      },
      onFailed: (listener) => {
        port1.addEventListener("message", ({ data }: MessageEvent<{ type?: string }>) => {
          if (data.type === "component-failed") listener(data as ComponentFailed);
        });
      },
    };
    port1.addEventListener("message", ({ data }: MessageEvent<{ type?: string }>) => {
      if (data.type === "source-link") void showPassage((data as SourceLink).fragment);
      // Only to another site, as the frame sends it: never a script or a page of the playground's own.
      if (data.type === "open-link" && /^(https?|mailto):/i.test((data as OpenLink).url)) open((data as OpenLink).url, "_blank", "noopener,noreferrer");
    });
    frame = connected;
    connected.send({ type: "host-style", style: hostStyle });
    const opened = new Editor(worker, connected, () => {
      shown = { ...opened.state };
    });
    editor = opened;
    if (example) text = opened.open(example);
    // A shared link selects its passage once, in the document it opened with.
    if (shared !== undefined) void showPassage(`:~:text=${shared}`);
    shared = undefined;
    // For the bench, which reads the timings.
    (window as { compmdEditor?: Editor }).compmdEditor = opened;
  }

  const ms = (value: number | undefined) => (value === undefined ? "–" : `${value.toFixed(1)} ms`);
</script>

<header>
  <div class="title">
    <picture class="logo">
      <source media="(prefers-color-scheme: dark)" srcset={markDark} />
      <img src={markLight} alt="" width="40" height="26" />
    </picture>
    <h1>CompMD Playground</h1>
    <button type="button" class="about-toggle" aria-label="About" title="About" aria-expanded={about} aria-controls="about" onclick={toggleAbout}>i</button>
  </div>
  <label class="document">
    <span>Document</span>
    <select bind:value={chosen} onchange={() => (file = undefined)}>
      {#each EXAMPLES as example, i (example.name)}
        <option value={i}>{example.name}</option>
      {/each}
    </select>
  </label>
  <label class="document" title="The page's look is its host's, not the document's: try the stylesheets a host might give it.">
    <span>Host stylesheet</span>
    <select bind:value={hostStyle}>
      {#each HOST_STYLES as { id, label } (id)}
        <option value={id}>{label}</option>
      {/each}
    </select>
  </label>
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
  <nav class="links" aria-label="CompMD">
    <a href="https://github.com/comp-markdown/compmd/blob/main/compmd-spec-v1.md" target="_blank" rel="noreferrer">Spec</a>
    <a href="https://github.com/comp-markdown/compmd" target="_blank" rel="noreferrer">GitHub</a>
  </nav>
</header>

{#if about}
  <section class="about" id="about" aria-label="About the playground">
    <p>
      Write a <a href="https://github.com/comp-markdown/compmd" target="_blank" rel="noreferrer">CompMD</a> document on the left and see it
      rendered on the right as you type: components, data, layout and all. The tabs show what a plain Markdown renderer makes of the same
      text, what the frontmatter gives the page, the components the document can name and where it uses them, and the trees the processor
      builds. Beside the document, try the stylesheets a host might give the page; above the tabs, other values for the document's inputs. Under the text,
      a line says what is at the caret and what it takes, with its values to click, and the diagnostics point at what's wrong and offer
      fixes. Nothing leaves your browser.
    </p>
    <button type="button" class="close" aria-label="Close" onclick={toggleAbout}>
      <svg viewBox="0 0 12 12" aria-hidden="true"><path d="m3 3 6 6m0-6-6 6" /></svg>
    </button>
  </section>
{/if}

<main>
  <Split text={source} views={panes} />
</main>

<footer class="statusbar">
  <span>Copyright © 2026 Hakan Serce.</span>
  <span>The editor is <a href="https://github.com/comp-markdown/compmd/blob/main/LICENSE-CODE" target="_blank" rel="noreferrer">MIT licensed</a>.</span>
</footer>

{#snippet source()}
  <div class="sheet text">
    {#if example && example.files.length > 0}
      <div class="tabs files" role="tablist" aria-label="The document and the files it reads">
        <button type="button" role="tab" aria-selected={file === undefined} onclick={() => (file = undefined)}>{example.request.path}</button>
        {#each example.files as supporting (supporting.name)}
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
      {#if selected && file === undefined}
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
              <button
                type="button"
                class="fix"
                onclick={() => write(editor?.fix(fix))}>Use <code>{fix.text}</code></button
              >
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
    <div class="tabs" role="tablist" aria-label="Views">
      {#each TABS as { id, label } (id)}
        <button type="button" role="tab" id="tab-{id}" aria-selected={tab === id} aria-controls="view-{id}" onclick={() => (tab = id)}>{label}</button>
      {/each}
    </div>
    <!-- The page stays mounted under the other tabs, so its components keep their state. -->
    <div class="view page-view" id="view-page" role="tabpanel" aria-labelledby="tab-page" hidden={tab !== "page"}>
      {#key chosen}
        <iframe title="The rendered page" sandbox="allow-scripts" src="./frame.html" onload={({ currentTarget }) => {
            if (currentTarget instanceof HTMLIFrameElement) start(currentTarget);
          }}></iframe>
      {/key}
    </div>
    {#if tab !== "page"}
      <div class="view" id="view-{tab}" role="tabpanel" aria-labelledby="tab-{tab}">
        {#if view === undefined || view.view !== kindOf(tab)}
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
