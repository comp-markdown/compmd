<!-- The site's header: the logo, home, and the three places. -->
<script lang="ts">
  import Logo from "./Logo.svelte";
  import { REPOSITORY } from "./site.ts";

  interface Props {
    /** The path from this page to the site's root: `""`, or `"../"` per folder down. */
    root?: string;
    /** The place this page is. */
    current?: "spec" | undefined;
  }

  let { root = "", current }: Props = $props();
</script>

<header class="site">
  <div class="wrap">
    <a class="home" href={root || "./"} aria-label="CompMD home"><Logo /></a>
    <nav aria-label="The site">
      <a href="{root}spec/" aria-current={current === "spec" ? "page" : undefined}>Spec</a>
      <a href="{root}playground/">Playground</a>
      <a href={REPOSITORY}>GitHub</a>
    </nav>
  </div>
</header>
