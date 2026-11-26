import type { Request } from "@compmd/processor";
import type { ViewEvents } from "@compmd/sandbox";
import { describe, expect, it } from "vitest";
import type { DevtoolsDocument } from "./document.ts";
import { Follower, type FollowedPage, type FollowingEditor } from "./follow.ts";

/** An editor that records what it is asked. */
class FakeEditor implements FollowingEditor {
  readonly asked: (["open", DevtoolsDocument] | ["follow", string])[] = [];
  open(document: DevtoolsDocument): string {
    this.asked.push(["open", document]);
    return "";
  }
  follow(text: string): void {
    this.asked.push(["follow", text]);
  }
}

/** A page that records the lines it scrolls to, and paints when the test says. */
class FakePage implements FollowedPage {
  readonly scrolled: number[] = [];
  #painted: (() => void) | undefined;
  on<T extends keyof ViewEvents>(type: T, listener: (event: ViewEvents[T]) => void): void {
    if (type === "rendered") {
      this.#painted = () => {
        listener({ ms: 1 } as ViewEvents[T]);
      };
    }
  }
  scrollToLine(line: number): void {
    this.scrolled.push(line);
  }
  paint(): void {
    this.#painted?.();
  }
}

const REQUEST: Request = { path: "a.comp.md", files: { "data.csv": "x\n1\n" }, now: "2026-11-26T00:00:00Z" };

/** A request the test gives when it likes, as the server would answer. */
function deferred() {
  const answers: ((request: Request) => void)[] = [];
  const failures: ((error: Error) => void)[] = [];
  const request = () =>
    new Promise<Request>((resolve, reject) => {
      answers.push(resolve);
      failures.push(reject);
    });
  return { request, answers, failures };
}

function setup() {
  const editor = new FakeEditor();
  const page = new FakePage();
  const failures: string[] = [];
  const server = deferred();
  const follower = new Follower(editor, page, server.request, (message) => failures.push(message));
  return { editor, page, failures, server, follower };
}

const opened = (text: string): ["open", DevtoolsDocument] => ["open", { request: { ...REQUEST, files: { ...REQUEST.files, "a.comp.md": text } } }];

describe("Follower", () => {
  it("opens the document with the server's request and the latest text, then follows each change", async () => {
    const { editor, server, follower } = setup();
    follower.change({ text: "# A", line: 1 });
    follower.change({ text: "# AB", line: 1 });
    expect(server.answers).toHaveLength(1);
    server.answers[0]?.(REQUEST);
    await Promise.resolve();
    expect(editor.asked).toEqual([opened("# AB")]);
    follower.change({ text: "# ABC", line: 1 });
    expect(editor.asked[1]).toEqual(["follow", "# ABC"]);
  });

  it("scrolls to the line once the page painted the change, and at once when only the line moved", async () => {
    const { page, server, follower } = setup();
    follower.change({ text: "# A\n\nText", line: 3 });
    server.answers[0]?.(REQUEST);
    await Promise.resolve();
    expect(page.scrolled).toEqual([]);
    page.paint();
    expect(page.scrolled).toEqual([3]);
    follower.change({ text: "# A\n\nText", line: 1 });
    expect(page.scrolled).toEqual([3, 1]);
  });

  it("opens again with the request the server gives after a reload, the earlier fetch stale", async () => {
    const { editor, server, follower } = setup();
    follower.reload();
    expect(server.answers).toHaveLength(0);
    follower.change({ text: "# A", line: 1 });
    follower.reload();
    const again = { ...REQUEST, files: { "data.csv": "x\n2\n" } };
    server.answers[1]?.(again);
    server.answers[0]?.(REQUEST);
    await Promise.resolve();
    expect(editor.asked).toEqual([["open", { request: { ...again, files: { "data.csv": "x\n2\n", "a.comp.md": "# A" } } }]]);
  });

  it("says why when the request can't be had, and asks again at the next change", async () => {
    const { editor, failures, server, follower } = setup();
    follower.change({ text: "# A", line: 1 });
    server.failures[0]?.(new Error("connection refused"));
    await Promise.resolve();
    await Promise.resolve();
    expect(failures).toEqual(["the document couldn't be had from the language server: connection refused"]);
    follower.change({ text: "# AB", line: 1 });
    server.answers[1]?.(REQUEST);
    await Promise.resolve();
    expect(editor.asked).toEqual([opened("# AB")]);
  });
});
