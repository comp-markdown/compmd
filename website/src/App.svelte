<!-- The landing page: one column of sections, each one idea with something to look at (website/docs/design.md). -->
<script lang="ts">
  import snapshots from "virtual:snapshots";
  import Degrade from "./Degrade.svelte";
  import Example from "./Example.svelte";
  import Footer from "./Footer.svelte";
  import Header from "./Header.svelte";
  import Hero from "./Hero.svelte";
  import SectionWheel from "./SectionWheel.svelte";
  import { REPOSITORY } from "./site.ts";
  import Snippets from "./Snippets.svelte";

  /** The page's sections, for the contents wheel. */
  const SECTIONS = [
    { id: "top", label: "CompMD" },
    { id: "why", label: "Why CompMD" },
    { id: "what", label: "What it adds" },
    { id: "plain", label: "Still Markdown" },
    { id: "markdown", label: "Plain Markdown too" },
    { id: "use", label: "Use it in your app" },
    { id: "safe", label: "Safe to open" },
    { id: "hood", label: "Under the hood" },
    { id: "next", label: "Where next" },
  ];

  const SPEC_SECURITY = "spec/#15-security";

  const css = Object.values(snapshots)
    .map((snapshot) => snapshot.css)
    .join("\n");

  const FEATURES = [
    {
      title: "Components and slots",
      text: "Define a component once, in CompMD itself: typed params, a body, and named slots the caller fills.",
      name: "card.comp.md",
      snapshot: snapshots.components,
    },
    {
      title: "Data",
      text: "Read CSV, JSON or YAML from the frontmatter, then filter, sort and add it up with expressions.",
      name: "books.comp.md",
      snapshot: snapshots.data,
    },
    {
      title: "Inputs",
      text: "Declare a value the reader changes, bind an input to it, and the page follows. Try it.",
      name: "reading.comp.md",
      snapshot: snapshots.inputs,
      live: "page" as const,
    },
    {
      title: "Layout",
      text: "Grids, rows and stacks that follow the width of the box they are in, not the window.",
      name: "plan.comp.md",
      snapshot: snapshots.layout,
    },
    {
      title: "Components of your own",
      text: "When CompMD isn't enough, write a component in JSX, TSX, Svelte or JavaScript. It runs in a sandbox, as this one does; a renderer that can't run it shows its fallback.",
      name: "meter.comp.md",
      snapshot: snapshots.host,
      live: "frame" as const,
      height: "7.5rem",
    },
  ];

  const PLACES = [
    { href: "spec/", title: "The spec", text: "Version 1, a draft: the syntax, the components, expressions, data and security." },
    { href: "playground/", title: "The playground", text: "Write CompMD and watch the page, with a tour, examples and the diagnostics." },
    { href: `${REPOSITORY}/tree/main/conformance`, title: "The conformance set", text: "177 cases any implementation can run, with their expected trees." },
    { href: REPOSITORY, title: "GitHub", text: "The spec, the processor in Rust, the packages and this site." },
    { href: `${REPOSITORY}/tree/main/wasm`, title: "The packages", text: "@compmd/processor and @compmd/render, to be published on npm." },
  ];
</script>

<svelte:head>
  {@html `<style>${css}</style>`}
</svelte:head>

<Header />

<SectionWheel sections={SECTIONS} />

<main>
  <section class="hero" id="top">
    <div class="wrap">
      <h1>Markdown with components.</h1>
      <p class="lede">Readable as text, rendered as a page. Boxes, grids, values from data and inputs the reader changes, written in lines that still read as Markdown.</p>
      <p class="actions">
        <a class="button primary" href="playground/">Open the playground</a>
        <a class="button" href="spec/">Read the spec</a>
      </p>
      <Hero />
    </div>
  </section>

  <section id="why">
    <div class="wrap">
      <h2>Why CompMD</h2>
      <div class="beats">
        <div>
          <h3>Markdown stops where pages start.</h3>
          <p>It has headings, lists and tables, but no way to put two things side by side, show a number from data, or let the reader change what the page shows.</p>
        </div>
        <div>
          <h3>The usual way out leaves Markdown behind.</h3>
          <p>MDX turns the document into a program that needs a build, and templates bury the text in markup. Either way, the file stops reading as text.</p>
        </div>
        <div>
          <h3>CompMD adds components that read as text.</h3>
          <p>A component is a line of its own, <code>::: box</code>, with its params in braces. A CompMD processor renders it; any other Markdown tool shows readable text.</p>
        </div>
      </div>
    </div>
  </section>

  <section id="what">
    <div class="wrap">
      <h2>What it adds</h2>
      <p class="lede">Each example is a document and the page it renders, made by the processor when this site was built.</p>
      {#each FEATURES as feature (feature.title)}
        <div class="feature">
          <div class="about">
            <h3>{feature.title}</h3>
            <p>{feature.text}</p>
          </div>
          <Example name={feature.name} snapshot={feature.snapshot} live={feature.live} height={feature.height} />
        </div>
      {/each}
    </div>
  </section>

  <section id="plain">
    <div class="wrap">
      <h2>Still Markdown</h2>
      <p class="lede">A CompMD file is a Markdown file. On GitHub, in an editor's preview or in any Markdown renderer it reads well: each component shows its fallback text, and nothing turns into noise. Switch the renderer to see.</p>
      <Degrade />
    </div>
  </section>

  <section id="markdown">
    <div class="wrap">
      <h2>Great for plain Markdown too</h2>
      <p class="lede">A document without a single component is a CompMD document, and gets everything the processor and the renderer give any document.</p>
      <div class="feature">
        <ul class="claims">
          <li>CommonMark and GFM as GitHub renders them: tables, task lists, footnotes and alerts.</li>
          <li>A live preview that renders only the blocks an edit changed, and keeps the rest of the page as it was.</li>
          <li>Every rendered block knows the lines it came from, so an editor can find the text behind the page.</li>
        </ul>
        <Example name="trip.md" snapshot={snapshots.plain} />
      </div>
    </div>
  </section>

  <section id="use">
    <div class="wrap">
      <h2>Use it in your app</h2>
      <p class="lede">Two packages: the processor, which turns text into a tree, and the renderer, which turns the tree into a page.</p>
      <pre class="install"><code>npm install @compmd/processor @compmd/render</code></pre>
      <p class="note">Not on npm yet. Until they are, the packages are in the repository: <a href="{REPOSITORY}/tree/main/wasm">wasm/</a> and <a href="{REPOSITORY}/tree/main/render">render/</a>.</p>
      <Snippets />
    </div>
  </section>

  <section id="safe">
    <div class="wrap">
      <h2>Safe to open</h2>
      <p class="lede">A CompMD document can come from anyone. Opening one can't run code in your page, reach your data, or call home.</p>
      <div class="columns">
        <div>
          <h3>The threat</h3>
          <p>A document from a stranger, or a pack it imports, that tries to run code in the reader's page, read the reader's data or session, send data out, run forever, or act in the reader's name.</p>
        </div>
        <div>
          <h3>What stops it</h3>
          <ul>
            <li>Components written in CompMD run no code at all: only Markdown, params and slots.</li>
            <li>Code in JSX, Svelte or JavaScript runs sandboxed. It sees only its params and slots, with no network, no storage or cookies, no navigation, and it is stopped when it runs too long.</li>
            <li>Data comes only from origins the reader or the host allows, never ones a document grants itself.</li>
            <li>Text is escaped, and raw HTML is limited to a short list of safe tags.</li>
            <li>Expressions run within a work budget, and expansion within a size limit, so no document can hang the page.</li>
          </ul>
        </div>
        <div>
          <h3>How this implementation builds it</h3>
          <ul>
            <li>The playground renders in a sandboxed frame with an opaque origin, under a content security policy that allows only its own files.</li>
            <li>The processor counts the work of each expression, limits the size of the expansion, and refuses recursion.</li>
          </ul>
          <p><a href={SPEC_SECURITY}>The spec's section 15</a> says what every implementation must do.</p>
        </div>
      </div>
    </div>
  </section>

  <section id="hood">
    <div class="wrap">
      <h2>Under the hood</h2>
      <dl class="points">
        <div>
          <dt>A processor in Rust, run as WebAssembly</dt>
          <dd>The same processor runs natively and in the browser, where it is 808 KB gzipped. <a href="{REPOSITORY}/tree/main/processor">The processor</a></dd>
        </div>
        <div>
          <dt>Incremental</dt>
          <dd>A keystroke in a 500 KB document is processed in 0.30 ms (p95, Chrome on an Apple M4): only the blocks an edit touches are processed again. <a href="{REPOSITORY}/blob/main/processor/docs/incremental.md">How</a></dd>
        </div>
        <div>
          <dt>Diagnostics with fixes, hints at the caret</dt>
          <dd>Every mistake has a code, a place and, where one is clear, the edit that fixes it. At the caret, the processor says what a name, param or path is and what may be written there.</dd>
        </div>
        <div>
          <dt>Expressions in CEL</dt>
          <dd>A profile of <a href="https://cel.dev">CEL</a>, implemented from scratch: type-checked before it runs, evaluated within a work budget, with CompMD's <code>md.</code> functions for dates, numbers and lists.</dd>
        </div>
        <div>
          <dt>A conformance set</dt>
          <dd>177 cases, with every diagnostic code, that any implementation can run against its own processor. <a href="{REPOSITORY}/tree/main/conformance">The cases</a></dd>
        </div>
      </dl>
    </div>
  </section>

  <section id="next">
    <div class="wrap">
      <h2>Where next</h2>
      <ul class="places">
        {#each PLACES as place (place.href)}
          <li>
            <a href={place.href}>
              <strong>{place.title}</strong>
              <span>{place.text}</span>
            </a>
          </li>
        {/each}
      </ul>
    </div>
  </section>
</main>

<Footer />
