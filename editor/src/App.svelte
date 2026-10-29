<!-- The editor: the document's text on the left, the rendered page on the right in a sandboxed frame, the diagnostics
     under the text. Each document opens with a worker and a frame of its own. -->
<script lang="ts">
  import { connectFrame } from "@compmd/render/frame";
  import { EXAMPLES } from "./documents.ts";
  import { Editor, type EditorState, type Frame } from "./editor.ts";
  import { FRAME_HELLO, type ComponentFailed } from "./protocol.ts";

  const initial = Number(new URLSearchParams(location.search).get("doc") ?? 0);
  let chosen = $state(Number.isInteger(initial) && EXAMPLES[initial] ? initial : 0);
  let text = $state("");
  let shown: EditorState | undefined = $state.raw();
  let textarea: HTMLTextAreaElement | undefined = $state();
  let editor: Editor | undefined;
  let worker: Worker | undefined;

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
          <li class={d.severity}><code>{d.code}</code> {d.file ? `${d.file} ` : ""}{d.range.start.line}:{d.range.start.column} {d.message}</li>
        {/each}
        {#each shown.problems as problem (problem)}
          <li class="error">{problem}</li>
        {/each}
      </ul>
    {/if}
  </section>
  {#key chosen}
    <iframe title="The rendered page" sandbox="allow-scripts" src="./frame.html" onload={(event) => start(event.currentTarget)}></iframe>
  {/key}
</main>
