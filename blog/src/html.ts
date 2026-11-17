// Text and attribute values written into the pages the blog writes, escaped.

const ENTITIES: Record<string, string> = { "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" };

/** `text` escaped for HTML or XML, as an element's text or an attribute's value. */
export const escape = (text: string): string => text.replace(/[&<>"']/g, (character) => ENTITIES[character] ?? character);
