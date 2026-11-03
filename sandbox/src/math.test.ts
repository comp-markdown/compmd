// @vitest-environment happy-dom
import { beforeAll, describe, expect, it } from "vitest";
import { typesetMath, type Typesetter } from "./math.ts";

// Temml refuses a page in quirks mode when it loads, and happy-dom has no `compatMode`: the frame's page, with its
// doctype, has this. So Temml loads after it, as the frame loads it, the first time a page has math.
let loaded: Typesetter;
beforeAll(async () => {
  Object.defineProperty(document, "compatMode", { value: "CSS1Compat", configurable: true });
  loaded = (await import("temml")).default;
});
const temml = (): Promise<Typesetter> => Promise.resolve(loaded);

const settled = () => new Promise((resolve) => setTimeout(resolve, 0));

describe("typesetMath", () => {
  it("shows the TeX until Temml has loaded, then typesets it as MathML, loading Temml once", async () => {
    let loads = 0;
    const math = typesetMath(() => {
      loads++;
      return temml();
    });
    // Inline only: display math sets the `style` of its `<math>`, which happy-dom's MathML elements lack.
    const first = document.createElement("span");
    const second = document.createElement("span");
    math("e^{i\\pi} + 1 = 0", { display: false }, first);
    math("\\frac{1}{2}", { display: false }, second);
    expect(first.textContent).toBe("e^{i\\pi} + 1 = 0");
    await settled();
    expect(first.firstElementChild?.tagName.toLowerCase()).toBe("math");
    expect(second.innerHTML).toBe("<math><mfrac><mn>1</mn><mn>2</mn></mfrac></math>");
    expect(loads).toBe(1);
  });

  it("shows the source of TeX it can't typeset", async () => {
    const math = typesetMath(temml);
    const target = document.createElement("span");
    math("\\frac{1}{", { display: false }, target);
    await settled();
    expect(target.textContent).toBe("\\frac{1}{");
    expect(target.querySelector("math")).toBeNull();
  });

  it("keeps the newest TeX when an older typesetting finishes after it", async () => {
    const rendered: string[] = [];
    const recording: Typesetter = {
      render: (tex, target) => {
        rendered.push(tex);
        target.textContent = `typeset ${tex}`;
      },
    };
    const math = typesetMath(() => Promise.resolve(recording));
    const target = document.createElement("span");
    math("a", { display: false }, target);
    math("b", { display: false }, target);
    await settled();
    expect(rendered).toEqual(["b"]);
    expect(target.textContent).toBe("typeset b");
  });

  it("leaves the source when Temml doesn't load", async () => {
    const math = typesetMath(() => Promise.reject(new Error("offline")));
    const target = document.createElement("span");
    math("x^2", { display: false }, target);
    await settled();
    expect(target.textContent).toBe("x^2");
  });
});
