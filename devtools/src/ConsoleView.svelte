<!-- The Console tab: what went wrong, newest last, each line with its time; what a frame reported is marked as its.
     Among them, the expressions evaluated in the document, each with its value or its F024; under them, the input for
     the next. -->
<script lang="ts">
  import type { Range } from "@compmd/processor";
  import { tick } from "svelte";
  import { ExpressionHistory, markedAt, withEvaluations, type ConsoleEntry, type Evaluation } from "./console.ts";
  import JsonTree from "./JsonTree.svelte";
  import { clockOf, frameName } from "./labels.ts";
  import Place from "./Place.svelte";
  import Reported from "./Reported.svelte";

  interface Props {
    entries: readonly ConsoleEntry[];
    /** The expressions evaluated so far, and how to evaluate the next; without `onevaluate`, there is no input. */
    evaluations?: readonly Evaluation[];
    onevaluate?: ((expression: string) => void) | undefined;
    /** `onselect` selects a diagnostic's place in the text; without it, as in the inspector, places are text. */
    onselect?: ((range: Range) => void) | undefined;
  }

  let { entries, evaluations = [], onevaluate, onselect }: Props = $props();

  /** The input's tooltip: what it takes, and where the language is described. */
  const EXPRESSIONS_HELP = [
    "An expression of the CompMD CEL profile, as in a param's parentheses: Enter evaluates it, Shift+Enter adds a line, ↑ and ↓ go through the earlier ones.",
    "The profile, its variables and its md. functions: the CompMD spec, section 8.2, https://compmd.hakanserce.com/spec/#82-expressions",
    "CEL itself, by example: https://celbyexample.com/",
  ].join("\n\n");

  /** Why the sandbox stopped a frame, as the line says it. */
  const STOPPED = {
    hung: "it stopped answering",
    version: "it is of another protocol version",
    unloaded: "its runtime never loaded",
  } as const;

  const lines = $derived(withEvaluations(entries, evaluations));
  const history = new ExpressionHistory();
  let written = $state("");
  let list: HTMLOListElement | undefined = $state();

  /** Enter evaluates, Shift+Enter adds a line; ↑ on the first line and ↓ on the last walk the history. */
  async function keydown(event: KeyboardEvent & { currentTarget: HTMLTextAreaElement }): Promise<void> {
    const area = event.currentTarget;
    const before = area.value.slice(0, area.selectionStart);
    const after = area.value.slice(area.selectionEnd);
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      const expression = written.trim();
      if (!expression || !onevaluate) return;
      history.add(expression);
      onevaluate(expression);
      written = "";
      await tick();
      list?.lastElementChild?.scrollIntoView({ block: "nearest" });
      return;
    }
    const walked = event.key === "ArrowUp" && !before.includes("\n") ? history.previous(written) : event.key === "ArrowDown" && !after.includes("\n") ? history.next() : undefined;
    if (walked === undefined) return;
    event.preventDefault();
    written = walked;
  }
</script>

<div class="console-pane">
  <ol class="console" aria-label="What went wrong and what was evaluated, newest last" bind:this={list}>
    {#each lines as line, i (i)}
      <li class={line.kind}>
        <time class="at">{clockOf(line.at)}</time>
        {#if line.kind === "evaluation"}
          {@const result = line.result}
          <div class="evaluated">
            {#if result !== undefined && "error" in result}
              <pre class="expression">{markedAt(line.expression, result.error.range.start.line, result.error.range.start.column)}</pre>
              <span class="error"><code class="code">{result.error.code}</code> {result.error.message}</span>
            {:else}
              <pre class="expression">{line.expression}</pre>
              {#if result === undefined}
                <span class="pending">Evaluating…</span>
              {:else if "value" in result}
                <div class="value"><JsonTree value={result.value} open={1} /></div>
              {:else}
                <span class="error">{result.failed}</span>
              {/if}
            {/if}
          </div>
        {:else if line.kind === "diagnostics"}
          {#if line.diagnostics.length === 0}
            <span>No diagnostics.</span>
          {:else}
            <span>{line.diagnostics.length === 1 ? "1 diagnostic" : `${String(line.diagnostics.length)} diagnostics`}:</span>
            <ul>
              {#each line.diagnostics as d, j (j)}
                <li class={d.severity}>
                  <code class="code">{d.code}</code>
                  {#if d.file === undefined}
                    <Place range={d.range} title="Select it in the text" {onselect}>{d.range.start.line}:{d.range.start.column}</Place>
                  {:else}
                    {d.file} {d.range.start.line}:{d.range.start.column}
                  {/if}
                  {d.message}
                </li>
              {/each}
            </ul>
          {/if}
        {:else if line.kind === "problem"}
          <span>{line.message}</span>
        {:else if line.kind === "fallback"}
          <span><code class="code">{line.code ?? line.reason}</code> <code>{line.component}</code> shows its fallback in {frameName(line.frame)}: {line.message}</span>
          <Reported by={line.by} />
        {:else if line.kind === "refused"}
          <span>A <code>set</code> of <code>{line.input}</code> from {frameName(line.frame)} was refused: none of its uses binds it.</span>
          <Reported by={line.by} />
        {:else}
          <span>{frameName(line.frame)} was stopped: {STOPPED[line.why]}.</span>
          <Reported by={line.by} />
        {/if}
      </li>
    {:else}
      <li class="empty">Nothing went wrong.</li>
    {/each}
  </ol>
  {#if onevaluate}
    <textarea
      class="evaluate"
      rows={written.split("\n").length}
      spellcheck="false"
      placeholder="An expression to evaluate in the document: md.doc, md.data.sales.size(), md.today()"
      title={EXPRESSIONS_HELP}
      aria-label="An expression to evaluate in the document. Enter evaluates it, Shift+Enter adds a line, the arrow keys go through the earlier ones."
      bind:value={written}
      onkeydown={(event) => void keydown(event)}
    ></textarea>
  {/if}
</div>
