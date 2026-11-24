// Text and attribute values written into the pages the blog writes, escaped.

const ENTITIES: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" };

/** `text` escaped for HTML or XML, as an element's text or an attribute's value. */
export const escape = (text: string): string => text.replace(/[&<>"']/g, (character) => ENTITIES[character] ?? character);

/**
 * JavaScript or JSON written inside a `<script>`, each `<` as `<`, which reads the same in a string, so no value in
 * it ends the element (`</script>`) or opens a comment (audit S29).
 */
export const inScript = (code: string): string => code.replace(/</g, "\\u003c");

/** CSS written inside a `<style>`, each `<` as CSS's escape for it, `\3c `, so nothing in it ends the element. */
export const inStyle = (css: string): string => css.replace(/</g, "\\3c ");
