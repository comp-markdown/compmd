// The preview's loop (docs/editor-support.md, decision 8): the writer's editor is elsewhere, and the server sends each
// change in it, the document's text and the line being edited. The first change opens the document with the request
// the server gives (its path, the files it reads, its policy); each one after it is an edit. Once the page painted it,
// the page scrolls to the block that holds the line. When what the document reads changes, the server says so, and the
// document opens again with the request it gives then.

import type { Request } from "@compmd/processor";
import type { View } from "@compmd/sandbox";
import type { Editor } from "./editor.ts";

/** A change in the writer's editor, as the server sends it. */
export interface WriterChange {
  /** The document's whole text. */
  text: string;
  /** The line being edited, from 1. */
  line: number;
}

/** What the loop uses of the editor: opening a document, and following its text. */
export type FollowingEditor = Pick<Editor, "open" | "follow">;

/** What the loop uses of the page: its paints, and scrolling to a line. */
export type FollowedPage = Pick<View, "on" | "scrollToLine">;

export class Follower {
  readonly #editor: FollowingEditor;
  readonly #page: FollowedPage;
  readonly #request: () => Promise<Request>;
  readonly #onFailure: (message: string) => void;
  /** The writer's text and line, as the last change gave them. */
  #text: string | undefined;
  #line: number | undefined;
  /** Whether the document is open in the editor, and which request is being fetched for it. */
  #opened = false;
  #fetching = 0;
  #fetched = 0;

  /**
   * Follows the writer's editor in `editor` and `page`; `request` gives the document's request but its text, from the
   * server, and `onFailure` hears why it couldn't be had.
   */
  constructor(editor: FollowingEditor, page: FollowedPage, request: () => Promise<Request>, onFailure: (message: string) => void) {
    this.#editor = editor;
    this.#page = page;
    this.#request = request;
    this.#onFailure = onFailure;
    page.on("rendered", () => {
      if (this.#line !== undefined) page.scrollToLine(this.#line);
    });
  }

  /**
   * The writer's editor changed: the document opens with its text the first time, and follows it after; the page
   * scrolls to `line` once it painted the change, or at once when the text is the same.
   */
  change({ text, line }: WriterChange): void {
    const same = text === this.#text;
    this.#text = text;
    this.#line = line;
    if (!this.#opened) {
      if (this.#fetching === this.#fetched) void this.#open();
    } else if (same) {
      this.#page.scrollToLine(line);
    } else {
      this.#editor.follow(text);
    }
  }

  /** What the document reads changed: it opens again, with the request the server gives now. */
  reload(): void {
    this.#opened = false;
    if (this.#text !== undefined) void this.#open();
  }

  /** Fetches the request and opens the document with the latest text; a later fetch makes this one stale. */
  async #open(): Promise<void> {
    const fetching = ++this.#fetching;
    let request: Request;
    try {
      request = await this.#request();
    } catch (error) {
      if (fetching === this.#fetching) this.#fetched = fetching;
      this.#onFailure(`the document couldn't be had from the language server: ${error instanceof Error ? error.message : String(error)}`);
      return;
    }
    if (fetching !== this.#fetching || this.#text === undefined) return;
    this.#fetched = fetching;
    this.#editor.open({ request: { ...request, files: { ...request.files, [request.path]: this.#text } } });
    this.#opened = true;
  }
}
