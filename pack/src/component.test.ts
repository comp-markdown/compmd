// `draw`, through the testing kit, as a pack's tests use both.
import { describe, expect, it } from "vitest";
import { draw } from "./component.ts";
import { mountForTest } from "./testing.ts";

/** A promise a test settles when it likes. */
function later<T>() {
  let resolve: (value: T) => void = () => undefined;
  let reject: (error: unknown) => void = () => undefined;
  const promise = new Promise<T>((yes, no) => {
    resolve = yes;
    reject = no;
  });
  return { promise, resolve, reject };
}

describe("draw", () => {
  it("draws SVG text or an element, with the role and the fallback's text as its label", async () => {
    const svg = mountForTest(
      draw<{ code: string }>(({ code }) => `<svg><text>${code}</text></svg>`, { role: "img" }),
      { params: { code: "A" }, fallback: "A diagram of A" },
    );
    await svg.settled();
    const drawing = svg.target.querySelector("[role=img]");
    expect(drawing?.getAttribute("aria-label")).toBe("A diagram of A");
    expect(drawing?.querySelector("text")?.textContent).toBe("A");
    const element = mountForTest(draw(() => Object.assign(document.createElement("b"), { textContent: "drawn" })));
    await element.settled();
    expect(element.target.textContent).toBe("drawn");
    expect(element.target.querySelector("[role]")).toBeNull();
    svg.destroy();
    element.destroy();
  });

  it("keeps the last drawing until the next is ready, and drops one a newer draw overtakes", async () => {
    const slow = later<string>();
    const fast = later<string>();
    const draws = [slow, fast];
    const chart = mountForTest(draw(() => draws.shift()?.promise ?? "third"), { params: { n: 1 } });
    chart.update({ n: 2 });
    fast.resolve("second");
    await chart.settled();
    expect(chart.target.textContent).toBe("second");
    slow.resolve("first");
    await slow.promise;
    await Promise.resolve();
    expect(chart.target.textContent).toBe("second");
    chart.update({ n: 3 });
    expect(chart.target.textContent).toBe("second");
    await chart.settled();
    expect(chart.target.textContent).toBe("third");
    chart.destroy();
  });

  it("fails with what the drawing threw, shows the fallback, and draws again at the next update", async () => {
    const chart = mountForTest(
      draw<{ code: string }>(({ code }) => {
        if (code === "A -->") throw new Error("Parse error on line 1");
        return `<svg><text>${code}</text></svg>`;
      }),
      { params: { code: "A -->" }, fallback: "A diagram" },
    );
    await chart.settled();
    expect(chart.failures).toEqual(["Parse error on line 1"]);
    expect(chart.target.textContent).toBe("A diagram");
    chart.update({ code: "A to B" });
    await chart.settled();
    expect(chart.target.textContent).toBe("A to B");
    chart.destroy();
  });

  it("draws again in the new scheme", async () => {
    const chart = mountForTest(draw((_, { scheme }) => scheme), { scheme: "dark" });
    await chart.settled();
    expect(chart.target.textContent).toBe("dark");
    chart.setScheme("light");
    await chart.settled();
    expect(chart.target.textContent).toBe("light");
    chart.destroy();
  });

  it("draws one at a time when asked, for a library with one global configuration", async () => {
    const order: string[] = [];
    const first = later<string>();
    const renderer = draw<{ name: string }>(
      async ({ name }) => {
        order.push(`start ${name}`);
        if (name === "one") await first.promise;
        order.push(`end ${name}`);
        return name;
      },
      { oneAtATime: true },
    );
    const one = mountForTest(renderer, { params: { name: "one" } });
    const two = mountForTest(renderer, { params: { name: "two" } });
    await Promise.resolve();
    expect(order).toEqual(["start one"]);
    first.resolve("");
    await two.settled();
    expect(order).toEqual(["start one", "end one", "start two", "end two"]);
    one.destroy();
    two.destroy();
  });

  it("doesn't fail for a draw that was overtaken or a component that went, and empties its box when it goes", async () => {
    const pending = later<string>();
    const chart = mountForTest(draw(() => pending.promise));
    const target = chart.target;
    chart.destroy();
    pending.reject(new Error("too late"));
    await chart.settled();
    expect(chart.failures).toEqual([]);
    expect(target.childNodes).toHaveLength(0);
  });
});
