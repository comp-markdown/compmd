// The testing kit, with plain components: the contract as the renderer runs it.
import type { Mount } from "@compmd/render";
import { describe, expect, it } from "vitest";
import { mountForTest } from "./testing.ts";

/** A counter: shows its `count`, and sets it one higher on a click. */
const counter: Mount = (target, { params, set, scheme }) => {
  const button = target.appendChild(document.createElement("button"));
  const show = (count: unknown) => {
    button.textContent = `${String(count)} (${scheme})`;
    button.onclick = () => {
      set("count", Number(count) + 1);
    };
  };
  show(params.count);
  return { update: (next) => { show(next.count); } };
};

describe("mountForTest", () => {
  it("gives the params, the scheme and the fallback, and records what the component sets", () => {
    const shown = mountForTest(counter, { params: { count: 1 }, scheme: "dark", inputs: ["count"] });
    expect(shown.target.textContent).toBe("1 (dark)");
    shown.target.querySelector("button")?.click();
    expect(shown.sets).toEqual([{ param: "count", value: 2 }]);
    shown.update({ count: 2 });
    expect(shown.target.textContent).toBe("2 (dark)");
    shown.destroy();
    expect(shown.target.isConnected).toBe(false);
  });

  it("gives the box's fit, contain and centred by default, and the new one with an update, the params kept", () => {
    const seen: string[] = [];
    const fitted: Mount = (_, context) => {
      seen.push(JSON.stringify(context.fit));
      return { update: (params) => seen.push(`${JSON.stringify(context.fit)} ${JSON.stringify(params)}`) };
    };
    const plain = mountForTest(fitted);
    const shown = mountForTest(fitted, { params: { n: 1 }, fit: { mode: "cover", x: 0, y: 1 } });
    shown.setFit({ mode: "fill", x: 1, y: 0.5 });
    expect(seen).toEqual([JSON.stringify({ mode: "contain", x: 0.5, y: 0.5 }), JSON.stringify({ mode: "cover", x: 0, y: 1 }), `${JSON.stringify({ mode: "fill", x: 1, y: 0.5 })} {"n":1}`]);
    plain.destroy();
    shown.destroy();
  });

  it("records a throw in update as a failure, and shows the fallback until the next update", async () => {
    const fragile: Mount = (target) => ({
      update: (next) => {
        if (next.bad === true) throw new Error("bad params");
        target.textContent = "fine";
      },
    });
    const shown = mountForTest(fragile, { fallback: "As text" });
    shown.update({ bad: true });
    expect(shown.failures).toEqual(["bad params"]);
    expect(shown.target.textContent).toBe("As text");
    shown.update({ bad: false });
    await shown.settled();
    expect(shown.target.textContent).toBe("fine");
    shown.destroy();
  });
});
