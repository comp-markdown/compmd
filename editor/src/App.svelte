<!-- The editor: the document's text on the left, with its diagnostics under it; on the right, tabs with the rendered
     page in a sandboxed frame and the document's trees. Each document opens with a worker and a frame of its own. -->
<script lang="ts">
  import type { Diagnostic } from "@compmd/processor";
  import { connectFrame } from "@compmd/render/frame";
  import { EXAMPLES } from "./documents.ts";
  import { offsetOf } from "./edit.ts";
  import { Editor, type EditorState, type Frame } from "./editor.ts";
  import { FRAME_HELLO, type ComponentFailed } from "./protocol.ts";
  import { TreesClient, type Trees } from "./trees.ts";

  const initial = Number(new URLSearchParams(location.search).get("doc") ?? 0);
  let chosen = $state(Number.isInteger(initial) && EXAMPLES[initial] ? initial : 0);
  let text = $state("");
  let shown: EditorState | undefined = $state.raw();
  let textarea: HTMLTextAreaElement | undefined = $state();
  let editor: Editor | undefined;
  let worker: Worker | undefined;

  const TABS = [
    { id: "page", label: "Page" },
    { id: "parse", label: "Parse tree" },
    { id: "expanded", label: "Expanded tree" },
  ] as const;
  type Tab = (typeof TABS)[number]["id"];
  let tab: Tab = $state("page");
  let trees: Trees | undefined = $state.raw();
  let treesClient: TreesClient | undefined;

  /** The trees of the document as it is now, while a tab shows them; the trees' worker starts the first time. */
  $effect(() => {
    if (tab === "page" || shown === undefined) return;
    void shown.updates;
    const document = editor?.document;
    if (!document) return;
    treesClient ??= new TreesClient(new Worker(new URL("./trees-worker.ts", import.meta.url), { type: "module" }), (answer) => {
      trees = answer;
    });
    // Without the origins the editor's keys need, which would make the expanded tree three times as long.
    treesClient.request({ ...document, origins: false });
  });

  /** Selects a diagnostic's range in the text area (to the end of its line, when it has no end), and scrolls to it. */
  function select({ range }: Diagnostic): void {
    if (!textarea) return;
    const start = offsetOf(textarea.value, range.start);
    const lineEnd = textarea.value.indexOf("\n", start);
    const end = range.end ? offsetOf(textarea.value, range.end) : lineEnd === -1 ? textarea.value.length : lineEnd;
    textarea.focus();
    textarea.setSelectionRange(start, end);
    const lineHeight = Number.parseFloat(getComputedStyle(textarea).lineHeight) || 20;
    textarea.scrollTop = Math.max(0, (range.start.line - 3) * lineHeight);
  }

  /** Opens the chosen document once its frame has loaded. */
  function start(iframe: HTMLIFrameElement): void {
    worker?.terminate();
    worker = new Worker(new URL("./worker.ts", import.meta.url), { type: "module" });
    const { port1, port2 } = new MessageChannel();
    iframe.contentWindow?.postMessage(FRAME_HELLO, "*", [port2]);
    const frame: Frame = {
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
    const opened = new Editor(worker, frame, () => {
      shown = { ...opened.state };
    });
    editor = opened;
    const example = EXAMPLES[chosen];
    if (example) text = opened.open(example);
    // For the bench, which reads the timings.
    (window as { compmdEditor?: Editor }).compmdEditor = opened;
  }

  const ms = (value: number | undefined) => (value === undefined ? "–" : `${value.toFixed(1)} ms`);
</script>

<header>
  <h1>CompMD editor</h1>
  <label>
    Document
    <select bind:value={chosen}>
      {#each EXAMPLES as example, i (example.name)}
        <option value={i}>{example.name}</option>
      {/each}
    </select>
  </label>
  <p class="status" aria-live="off">
    processing {ms(shown?.processing.last)} (p95 {ms(shown?.processing.p95)}) · key press to paint {ms(shown?.painted.last)} (p95 {ms(shown?.painted.p95)})
  </p>
</header>

<main>
  <section class="text">
    <textarea
      bind:this={textarea}
      value={text}
      spellcheck="false"
      aria-label="The document's text"
      onbeforeinput={() => {
        if (textarea) editor?.beforeInput({ start: textarea.selectionStart, end: textarea.selectionEnd });
      }}
      oninput={(event) => {
        if (textarea) editor?.input(textarea.value, event.timeStamp);
      }}
    ></textarea>
    {#if shown && (shown.diagnostics.length > 0 || shown.problems.length > 0)}
      <ul class="diagnostics" aria-label="Diagnostics">
        {#each shown.diagnostics as d, i (i)}
          <li class={d.severity}>
            <code>{d.code}</code>
            {#if d.file === undefined}
              <button type="button" class="place" title="Select it in the text" onclick={() => select(d)}>{d.range.start.line}:{d.range.start.column}</button>
            {:else}
              {d.file} {d.range.start.line}:{d.range.start.column}
            {/if}
            {d.message}
            {#if d.fix && d.file === undefined}
              {@const fix = d.fix}
              <button
                type="button"
                onclick={() => {
                  if (textarea && editor) textarea.value = editor.fix(fix);
                }}>Use <code>{fix.text}</code></button
              >
            {/if}
          </li>
        {/each}
        {#each shown.problems as problem (problem)}
          <li class="error">{problem}</li>
        {/each}
      </ul>
    {/if}
  </section>
  <section class="views">
    <div class="tabs" role="tablist" aria-label="Views">
      {#each TABS as { id, label } (id)}
        <button type="button" role="tab" id="tab-{id}" aria-selected={tab === id} aria-controls="view-{id}" onclick={() => (tab = id)}>{label}</button>
      {/each}
    </div>
    <!-- The page stays mounted under the other tabs, so its components keep their state. -->
    <div class="view" id="view-page" role="tabpanel" aria-labelledby="tab-page" hidden={tab !== "page"}>
      {#key chosen}
        <iframe title="The rendered page" sandbox="allow-scripts" src="./frame.html" onload={(event) => start(event.currentTarget)}></iframe>
      {/key}
    </div>
    {#if tab === "parse" || tab === "expanded"}
      <div class="view" id="view-{tab}" role="tabpanel" aria-labelledby="tab-{tab}">
        {#if trees === undefined}
          <p class="note">Processing…</p>
        {:else if "failed" in trees}
          <p class="note error">{trees.failed}</p>
        {:else}
          <pre class="tree">{tab === "parse" ? trees.parse : trees.expanded}</pre>
        {/if}
      </div>
    {/if}
  </section>
</main>
