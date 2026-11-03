<!-- The site's header: the logo, home, the places (the spec, the guides, the playground, GitHub), and search, which
     `search.ts` opens. -->
<script lang="ts">
  import Logo from "./Logo.svelte";
  import { REPOSITORY } from "./site.ts";

  interface Props {
    /** The path from this page to the site's root: `""`, or `"../"` per folder down. */
    root?: string;
    /** The place this page is. */
    current?: "spec" | "guides" | undefined;
  }

  let { root = "", current }: Props = $props();
</script>

<header class="site">
  <div class="wrap">
    <a class="home" href={root || "./"} aria-label="CompMD home"><Logo /></a>
    <nav aria-label="The site">
      <a href="{root}spec/" aria-current={current === "spec" ? "page" : undefined}>Spec</a>
      <a href="{root}guides/" aria-current={current === "guides" ? "page" : undefined}>Guides</a>
      <a href="{root}playground/">Playground</a>
      <a class="github" href={REPOSITORY}>GitHub</a>
      <button type="button" class="search" data-root={root} aria-label="Search the site" title="Search (/)">
        <svg viewBox="0 0 20 20" aria-hidden="true"><circle cx="8.5" cy="8.5" r="5.5" /><path d="m12.5 12.5 4.5 4.5" /></svg>
      </button>
    </nav>
  </div>
</header>
