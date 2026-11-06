<!-- The Developer Tools' inspector (docs/devtools.md, decision 3): attached to a sandbox view a host app already
     renders, with its document. No text and no page of its own: the CompMD Inspector's tabs alone (`Panes`). It observes the host's view from when it
     mounts, and stops when it goes; the views' worker processes the document the host gives, for its views and its
     diagnostics. -->
<script lang="ts">
  import type { Diagnostic } from "@compmd/processor";
  import type { ObservedEvent, View } from "@compmd/sandbox";
  import { untrack } from "svelte";
  import { consoleOf, EditorHistory } from "./console.ts";
  import type { DevtoolsDocument } from "./document.ts";
  import Panes from "./Panes.svelte";
  import { onceAFrame } from "./schedule.ts";

  let { view, document: first }: { view: View; document: DevtoolsDocument } = $props();

  /** The document as the host last gave it, and how many times it has. */
  let current = $state.raw(untrack(() => first));
  let revision = $state(0);

  /** The host's document again, as it processed it again: a new text, new inputs. */
  export function open(document: DevtoolsDocument): void {
    current = document;
    revision += 1;
  }

  let observed: readonly ObservedEvent[] = $state.raw([]);
  let dropped: Readonly<Record<string, number>> = $state.raw({});
  // Read in the next frame: the view hands its snapshot to the listener while `observe` runs.
  const readLater = onceAFrame(() => {
    observed = observation.events;
    dropped = observation.dropped;
  });
  const observation = untrack(() =>
    view.observe(() => {
      readLater();
    }),
  );
  readLater();
  $effect(() => () => {
    observation.stop();
  });

  /** The diagnostics as each version of the document had them, for the console. */
  const history = new EditorHistory();
  let noted = $state(0);
  function diagnosed(diagnostics: Diagnostic[]): void {
    history.note(diagnostics, []);
    noted += 1;
  }
  const entries = $derived.by(() => {
    void noted;
    return consoleOf(history.entries, observed);
  });
</script>

<div class="devtools inspector">
  <Panes document={current.request} {revision} {observed} {dropped} {entries} ondiagnostics={diagnosed} text={current.request.files?.[current.request.path]} />
</div>
