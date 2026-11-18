import { describe, expect, it } from "vitest";
import { parse, type Part } from "./parse.ts";

/** A part as the tests read it: its name, its texts, and its children. */
const shape = (part: Part): unknown => {
  const texts = part.positionals.map((value) => (value.kind === "string" ? value.text : value));
  return part.children.length === 0 ? [part.name, ...texts] : [part.name, ...texts, part.children.map(shape)];
};

/** The errors of `source`, as their messages. */
const errorsOf = (source: string): string[] => parse(source).errors.map((error) => error.message);

// The language's examples (LANGUAGE.md).
const SIGN_IN = `window "Sign in":
  header:
    text "Welcome back" bold
  panel:
    input placeholder="Email" id=email
    input placeholder="Password" type=password
    row justify=between:
      checkbox "Remember me"
      button "Sign in" primary
annotation "Shows an error under the field that failed" target="email" position=right
`;

const SETTINGS = `window "Settings":
  tabs:
    tab "General" active
    tab "Privacy"
    tab "Advanced"
  section "Notifications":
    toggle "Email me about replies" on
    checkbox "Weekly summary" checked
  section "Theme":
    radio "Light" group="theme"
    radio "Dark" group="theme" selected
    radio "Follow the system" group="theme"
  footer:
    row justify=end:
      button "Cancel"
      button "Save" primary
`;

const FILES = `window "Files":
    menubar:
        menu "File":
            menuitem "New folder" shortcut="⌘N"
            separator
            menuitem "Close" shortcut="⌘W"
        menu "Edit"
    row:
        col 220:
            tree:
                node "Documents" selected:
                    node "Reports"
                    node "Drafts" collapsed
                node "Pictures"
        col fill:
            grid cols=3 rows=2:
                cell "report.pdf" row=1 col=1
                cell "notes.txt" row=1 col=2
                cell "photo.png" row=1 col=3
`;

const PHONE = `# A phone's checkout
window "Order" device=phone:
  header:
    text "Your basket" bold

  list:
    item "Coffee beans, 1 kg" chevron
    item "Filter papers" chevron
  progress value=2 max=3 label="Step 2 of 3"
  button "Pay now" primary id=pay
annotation "Disabled until\\nan address is chosen" target="pay" position=bottom
`;

describe("parse, on the language's examples", () => {
  it("reads the sign-in screen: its parts, their attributes and flags, and the annotation", () => {
    const { wireframe, errors } = parse(SIGN_IN);
    expect(errors).toEqual([]);
    expect(shape(wireframe!.window)).toEqual([
      "window",
      "Sign in",
      [
        ["header", [["text", "Welcome back"]]],
        ["panel", [["input"], ["input"], ["row", [["checkbox", "Remember me"], ["button", "Sign in"]]]]],
      ],
    ]);
    const [, panel] = wireframe!.window.children;
    expect(panel!.children[0]!.attributes).toEqual({ placeholder: { kind: "string", text: "Email" }, id: { kind: "word", text: "email" } });
    expect(panel!.children[2]!.children[1]!.flags).toEqual(["primary"]);
    expect(wireframe!.annotations.map((note) => [note.positionals[0], note.attributes.position])).toEqual([[{ kind: "string", text: "Shows an error under the field that failed" }, { kind: "word", text: "right" }]]);
  });

  it("reads the settings, the file manager in 4-space indentation, and the phone screen with a comment and a blank line", () => {
    expect(parse(SETTINGS).errors).toEqual([]);
    const files = parse(FILES);
    expect(files.errors).toEqual([]);
    const [, row] = files.wireframe!.window.children;
    expect(row!.children[0]!.positionals).toEqual([{ kind: "number", value: 220, unit: "" }]);
    expect(row!.children[1]!.flags).toEqual(["fill"]);
    expect(shape(row!.children[0]!.children[0]!)).toEqual(["tree", [["node", "Documents", [["node", "Reports"], ["node", "Drafts"]]], ["node", "Pictures"]]]);
    const phone = parse(PHONE);
    expect(phone.errors).toEqual([]);
    expect(phone.wireframe!.window.attributes.device).toEqual({ kind: "word", text: "phone" });
    expect(phone.wireframe!.annotations[0]!.positionals[0]).toEqual({ kind: "string", text: "Disabled until\nan address is chosen" });
  });

  it("reads Wireloom's other parts, values and escapes", () => {
    const source = 'window:\n  navbar:\n    leading:\n      backbutton "Back"\n  panel:\n    slider range=0-100 value=40 label="Volume"\n    kv "Plan" "Pro" bold\n    text "She said \\"hi\\" \\\\ bye"\n    col 30%:\n      chart kind=pie\n';
    const { wireframe, errors } = parse(source);
    expect(errors).toEqual([]);
    const panel = wireframe!.window.children[1]!;
    expect(panel.children[0]!.attributes.range).toEqual({ kind: "range", from: 0, to: 100 });
    expect(panel.children[2]!.positionals).toEqual([{ kind: "string", text: 'She said "hi" \\ bye' }]);
    expect(panel.children[3]!.positionals).toEqual([{ kind: "number", value: 30, unit: "%" }]);
  });
});

describe("parse, on what isn't valid", () => {
  it("places each error at its line and column, says what to write, and gives no wireframe", () => {
    const { wireframe, errors } = parse('window "A":\n\tbutton "Go"\n  butn "Go"\n  spacer\n');
    expect(wireframe).toBeUndefined();
    expect(errors.map((error) => [error.line, error.column, error.problem])).toEqual([
      [2, 1, "a tab in the indentation: indent with 2 or 4 spaces"],
      [3, 3, "no part `butn`: did you mean `button`?"],
      [4, 3, "`spacer` is only in a `row`"],
    ]);
    expect(errors[0]!.message).toBe("line 2, column 1: a tab in the indentation: indent with 2 or 4 spaces");
  });

  it("checks the indentation", () => {
    expect(errorsOf('window:\n   text "x"\n')).toEqual(["line 2, column 1: the first indented line sets the indentation, 2 or 4 spaces, not 3"]);
    expect(errorsOf('window:\n  panel:\n     text "x"\n')).toEqual(["line 3, column 1: indented by 5 spaces: this source indents by 2"]);
    expect(errorsOf('window:\n  panel:\n      text "x"\n')).toEqual(["line 3, column 1: indented more than one level below the line before"]);
    expect(errorsOf('window:\n  panel\n    text "x"\n')).toEqual(["line 2, column 3: `panel` has children: end its line with `:`"]);
    expect(errorsOf('window:\n  text "x":\n    text "y"\n')).toEqual(["line 2, column 3: `text` takes no children: remove the `:`"]);
  });

  it("checks each part's arguments", () => {
    expect(errorsOf('window:\n  text\n  button "Go" primry\n  input size=3\n  row justify=middle:\n  grid cols=2:\n  slider range=9-1 value=4\n')).toEqual([
      "line 2, column 3: `text` needs its text, a quoted string",
      "line 3, column 15: `button` has no flag `primry`: did you mean `primary`?",
      "line 4, column 9: `input` has no attribute `size`",
      "line 5, column 7: `justify` is one of `start`, `between`, `around`, `end`, not `middle`",
      "line 6, column 3: `grid` needs `rows=`",
      "line 7, column 3: `range` is from the smaller number to the larger one",
    ]);
    expect(errorsOf('window:\n  text "a" "b"\n  kv "only"\n  checkbox "x" checked checked\n  col "wide":\n')).toEqual([
      "line 2, column 12: `text` takes at most 1 string here: `\"b\"`",
      "line 3, column 3: `kv` needs its two strings",
      "line 4, column 24: `checked` is given twice",
      "line 5, column 7: `col`'s width is a number, `240` or `30%`",
    ]);
    expect(errorsOf('window:\n  text "unended\n  text "\\t"\n  text "x" =\n')).toEqual([
      "line 2, column 8: a string that doesn't end: close it with `\"` on the same line",
      "line 3, column 9: `\\t` isn't an escape: `\\n`, `\\\"` and `\\\\` are",
      "line 4, column 12: `=` can't start an argument: write a string, a number, `name=value` or a flag",
    ]);
  });

  it("checks where parts go", () => {
    expect(errorsOf('window:\n  tabs:\n    button "x"\n  tab "y"\n  tree:\n    node "a":\n      node "b"\n')).toEqual([
      "line 3, column 5: `tabs` takes only `tab`, not `button`",
      "line 4, column 3: `tab` is only in a `tabs`",
    ]);
    expect(errorsOf('window:\n  header:\n  navbar:\n  footer:\n  tabbar:\n  sheet:\n  sheet:\n  segmented:\n')).toEqual([
      "line 3, column 3: a window has a `header` or a `navbar`, not both",
      "line 5, column 3: a window has a `footer` or a `tabbar`, not both",
      "line 7, column 3: a window has at most one `sheet`",
      "line 8, column 3: `segmented` goes in a row or a panel, not straight in the window",
    ]);
    expect(errorsOf('window:\n  navbar:\n    leading:\n    leading:\n  row:\n    segmented:\n      segment "a" selected\n      segment "b" selected\n')).toEqual([
      "line 4, column 5: a navbar has one `leading:`: put its parts in it",
      "line 8, column 7: one segment is `selected`",
    ]);
  });

  it("checks the window and the annotations", () => {
    expect(errorsOf("")).toEqual(["line 1, column 1: a source is one `window`, with its parts indented inside it"]);
    expect(errorsOf('panel:\nwindow:\nwindow:\n')).toEqual(["line 1, column 1: a source is one `window`; `panel` goes indented inside it", "line 3, column 1: a source has one `window`"]);
    expect(errorsOf('annotation "a" target="x" position=left\nwindow:\n')).toEqual(["line 1, column 1: an annotation comes after the window", "line 1, column 1: no part has the id `x` this annotation points at", "line 2, column 1: the window comes before the annotations"]);
    expect(errorsOf('window:\n  annotation "a"\nannotation "b" position=up\n')).toEqual([
      "line 2, column 3: an annotation goes at the top level, after the window, not inside it",
      "line 3, column 1: `annotation` needs `target=`",
      "line 3, column 16: `position` is one of `left`, `right`, `top`, `bottom`, not `up`",
    ]);
  });
});
