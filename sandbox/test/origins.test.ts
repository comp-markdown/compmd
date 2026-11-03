// Asking the reader before data from a new origin (spec §8.6), in the host's page: asked once, remembered, a configured
// origin never asked about, a denied one F028 with its fallback, and the plain dialog.

import { init, process } from "@compmd/processor";
import { afterEach, beforeAll, describe, expect, it } from "vitest";
import { allowOrigins, askWithDialog, loadData, localAnswers, type AskOrigin, type OriginAnswer } from "../src/index.ts";
import { allowed, denied, frames, hits, tagged } from "./harness.ts";

beforeAll(async () => {
  await init();
});

afterEach(() => {
  localStorage.clear();
});

/** A reader who answers `answer` each time, and how often they were asked, about which origins. */
function reader(answer: OriginAnswer) {
  const asked: string[] = [];
  const ask: AskOrigin = (origin) => {
    asked.push(origin);
    return new Promise((resolve) => {
      setTimeout(() => {
        resolve(answer);
      }, 10);
    });
  };
  return { ask, asked };
}

describe("allowOrigins", () => {
  it("asks once about a new origin, however many renders need it at once, and remembers the answer", async () => {
    const { ask, asked } = reader("allow");
    const urls = ["https://api.example.com/a.json", "https://api.example.com/b.json"];
    const both = await Promise.all([allowOrigins(urls, { configured: [], document: "a.comp.md", ask }), allowOrigins(urls, { configured: [], document: "a.comp.md", ask })]);
    expect(both).toEqual([["https://api.example.com"], ["https://api.example.com"]]);
    expect(asked).toEqual(["https://api.example.com"]);
    // Another document, another visit: the answer is kept, per origin.
    expect(await allowOrigins(urls, { configured: [], document: "b.comp.md", ask, answers: localAnswers() })).toEqual(["https://api.example.com"]);
    expect(asked).toHaveLength(1);
    expect(localAnswers().list()).toEqual([["https://api.example.com", "allow"]]);
  });

  it("never asks about an origin the host configured", async () => {
    const { ask, asked } = reader("deny");
    const allowed = await allowOrigins(["https://status.example.org/s.json"], { configured: ["https://status.example.org"], document: "a.comp.md", ask });
    expect(allowed).toEqual(["https://status.example.org"]);
    expect(asked).toEqual([]);
  });

  it("asks again once the answer is forgotten", async () => {
    const { ask, asked } = reader("deny");
    await allowOrigins(["https://api.example.com/a.json"], { configured: [], document: "a.comp.md", ask });
    localAnswers().forget("https://api.example.com");
    await allowOrigins(["https://api.example.com/a.json"], { configured: [], document: "a.comp.md", ask });
    expect(asked).toHaveLength(2);
  });

  it("leaves a denied origin out, so the processor reports F028 and loads the fallback", async () => {
    const { ask } = reader("deny");
    const text = "---\nmd:\n  data:\n    - from: https://api.example.com/n.json\n      as: n\n      fallback: ./cached.json\n---\n\n{{md.data.n.v}}\n";
    const request = { path: "a.comp.md", files: { "a.comp.md": text, "cached.json": '{"v": "cached"}' }, now: "2026-11-03T09:00:00Z" };
    const allowedOrigins = await allowOrigins(["https://api.example.com/n.json"], { configured: [], document: "a.comp.md", ask });
    const processed = process({ ...request, allowedOrigins });
    expect(processed.diagnostics.map(({ code }) => code)).toEqual(["F028"]);
    expect(JSON.stringify(processed.expanded)).toContain("cached");
  });
});

describe("loadData", () => {
  /** A document of one remote import from `url`, with a fallback. */
  const documentFrom = (url: string) => ({
    path: "a.comp.md",
    files: { "a.comp.md": `---\nmd:\n  data:\n    - from: ${url}\n      as: n\n      fallback: ./cached.json\n---\n\n[{{md.data.n}}]\n`, "cached.json": '"cached"' },
    now: "2026-11-03T09:00:00Z",
  });

  it("fetches from the origins the reader allows, without credentials or a Referer, and nothing from those they deny", async () => {
    const tag = tagged();
    const yes = `${allowed}${tag}.json`;
    const no = `${denied}${tag}.json`;
    const ask: AskOrigin = (origin) => Promise.resolve(yes.startsWith(origin) ? "allow" : "deny");
    const asked: RequestInit[] = [];
    const recordingFetch: typeof fetch = (input, init) => {
      asked.push(init ?? {});
      return fetch(input, init);
    };
    const loaded = await loadData([yes, no], { configured: [], document: "a.comp.md", ask, fetch: recordingFetch });
    expect(loaded.allowedOrigins).toEqual([new URL(allowed).origin]);
    expect(Object.keys(loaded.remote)).toEqual([yes]);
    expect(loaded.remote[yes]?.text).toBe("probed");
    expect(asked).toEqual([{ credentials: "omit", referrerPolicy: "no-referrer" }]);
    expect(await hits(allowed, tag)).toHaveLength(1);
    expect(await hits(denied, tag)).toEqual([]);
  });

  it("gives the processor what a render needs: the data from an allowed origin, the fallback where a fetch failed or the reader denied", async () => {
    const tag = tagged();
    const ask: AskOrigin = (origin) => Promise.resolve(origin === new URL(denied).origin ? "deny" : "allow");
    const render = async (url: string) => {
      const request = documentFrom(url);
      const loaded = await loadData([url], { configured: [], document: request.path, ask });
      const processed = process({ ...request, ...loaded });
      return { codes: processed.diagnostics.map(({ code }) => code), text: JSON.stringify(processed.expanded) };
    };
    // The frames' server answers 404 for a file it hasn't: a failed fetch uses the fallback, with no diagnostic (§8.6).
    const failed = await render(`${frames}${tag}.json`);
    expect(failed.codes).toEqual([]);
    expect(failed.text).toContain("[cached]");
    const refused = await render(`${denied}${tag}.json`);
    expect(refused.codes).toEqual(["F028"]);
    expect(refused.text).toContain("[cached]");
  });

  it("fetches nothing but from the configured origins when no reader can be asked", async () => {
    const tag = tagged();
    const deny: AskOrigin = () => Promise.resolve("deny");
    const loaded = await loadData([`${allowed}${tag}.json`], { configured: [], document: "a.comp.md", ask: deny });
    expect(loaded).toEqual({ allowedOrigins: [], remote: {} });
    expect(await hits(allowed, tag)).toEqual([]);
  });
});

describe("askWithDialog", () => {
  it("asks in a dialog of the host's page, saying what the origin learns, and gives the button pressed", async () => {
    const answer = askWithDialog("https://api.example.com", "a.comp.md");
    const dialog = document.querySelector<HTMLDialogElement>("dialog.compmd-origin-question");
    expect(dialog?.open).toBe(true);
    expect(dialog?.textContent).toContain("This document wants to load data from api.example.com. Allow?");
    expect(dialog?.textContent).toContain("learns the values you picked");
    const allow = [...(dialog?.querySelectorAll("button") ?? [])].find((button) => button.value === "allow");
    allow?.click();
    expect(await answer).toBe("allow");
    expect(document.querySelector("dialog.compmd-origin-question")).toBeNull();
  });

  it("is Don't allow when the reader closes it another way", async () => {
    const answer = askWithDialog("https://api.example.com", "a.comp.md");
    document.querySelector<HTMLDialogElement>("dialog.compmd-origin-question")?.close();
    expect(await answer).toBe("deny");
  });
});
