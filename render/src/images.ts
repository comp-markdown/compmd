// A document's images, for a page that can't load them itself: a page in a frame without the network gets each image's
// file from its host, which fetches only the images of the trees it sent.
import { htmlImageSources, safeUrl } from "./html.ts";
import { textOf, walk, type ExpandedNode } from "./tree.ts";

/**
 * Where a page's images come from: the URL to show for an image's `src`, as the document has it, or `undefined` when
 * it can't be had. Its alt text shows until then, and stays when it can't.
 */
export type ImageSource = (src: string) => Promise<string | undefined>;

/**
 * The `src` of every image `nodes` can show, as the renderer asks an {@link ImageSource} for it: Markdown images, link
 * reference definitions (which images may name), and images in raw HTML.
 *
 * @example
 * imageSources(tree.children); // ["./logo.png", "https://example.com/photo.png"]
 */
export function imageSources(nodes: readonly ExpandedNode[]): string[] {
  const sources: string[] = [];
  walk(nodes, (node) => {
    if (node.type === "html") sources.push(...htmlImageSources(textOf(node.value)));
    if (node.type !== "image" && node.type !== "definition") return;
    const src = safeUrl(textOf(node.url));
    if (src !== undefined) sources.push(src);
  });
  return sources;
}
