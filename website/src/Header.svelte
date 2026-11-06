<!-- The site's header: the logo, home, the places (the spec, the guides, the playground, GitHub), and search, which
     `search.ts` opens. -->
<script lang="ts">
  import Logo from "./Logo.svelte";
  import { REPOSITORY } from "./site.ts";

  interface Props {
    /** The path from this page to the site's root: `""`, or `"../"` per folder down. */
    root?: string;
    /** The place this page is. */
    current?: "spec" | "guides" | "playground" | undefined;
  }

  let { root = "", current }: Props = $props();
</script>

<header class="site">
  <div class="wrap">
    <a class="home" href={root || "./"} aria-label="CompMD home"><Logo /></a>
    <nav aria-label="The site">
      <a href="{root}spec/" aria-current={current === "spec" ? "page" : undefined}>Spec</a>
      <a href="{root}guides/" aria-current={current === "guides" ? "page" : undefined}>Guides</a>
      <a href="{root}playground/" aria-current={current === "playground" ? "page" : undefined}>Playground</a>
      <button type="button" class="search" data-root={root} aria-label="Search the site" title="Search (/)">
        <svg viewBox="0 0 20 20" aria-hidden="true"><circle cx="8.5" cy="8.5" r="5.5" /><path d="m12.5 12.5 4.5 4.5" /></svg>
      </button>
      <!-- Out of the site, so not a place among the others: GitHub's mark, as quiet as search. -->
      <a class="github" href={REPOSITORY} aria-label="CompMD on GitHub" title="CompMD on GitHub">
        <svg viewBox="0 0 16 16" aria-hidden="true">
          <path
            d="M8 0c4.42 0 8 3.58 8 8a8.013 8.013 0 0 1-5.45 7.59c-.4.08-.55-.17-.55-.38 0-.27.01-1.13.01-2.2 0-.75-.25-1.23-.54-1.48 1.78-.2 3.65-.88 3.65-3.95 0-.88-.31-1.59-.82-2.15.08-.2.36-1.02-.08-2.12 0 0-.67-.22-2.2.82-.64-.18-1.32-.27-2-.27-.68 0-1.36.09-2 .27-1.53-1.03-2.2-.82-2.2-.82-.44 1.1-.16 1.92-.08 2.12-.51.56-.82 1.28-.82 2.15 0 3.06 1.86 3.75 3.64 3.95-.23.2-.44.55-.51 1.07-.46.21-1.61.55-2.33-.66-.15-.24-.6-.83-1.23-.82-.67.01-.27.38.01.53.34.19.73.9.82 1.13.16.45.68 1.31 2.69.94 0 .67.01 1.3.01 1.49 0 .21-.15.45-.55.38A7.995 7.995 0 0 1 0 8c0-4.42 3.58-8 8-8Z"
          />
        </svg>
      </a>
    </nav>
  </div>
</header>
