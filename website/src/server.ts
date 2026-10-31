// The site's pages rendered on the server, for the prerender (`scripts/prerender.ts`).
import type { ComponentProps } from "svelte";
import { render } from "svelte/server";
import type { Rendered } from "../build/prerender.ts";
import App from "./App.svelte";
import Guide from "./Guide.svelte";
import Spec from "./Spec.svelte";

/** The landing page, which the client hydrates. */
export function renderPage(): Rendered {
  const { head, body } = render(App);
  return { head, body };
}

/** A version of the spec, static: nothing hydrates it. */
export function renderSpecPage(props: ComponentProps<typeof Spec>): Rendered {
  const { head, body } = render(Spec, { props });
  return { head, body };
}

/** A page of the guides, static: nothing hydrates it. */
export function renderGuidePage(props: ComponentProps<typeof Guide>): Rendered {
  const { head, body } = render(Guide, { props });
  return { head, body };
}
