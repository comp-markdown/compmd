// The book club's shelf: a cover for each book, drawn with d3 from its title and author. A click on a cover picks it,
// setting the input `picked` is bound to, and lifts it; the book being read says "This month", the next one is an
// outline. The covers share one row, each at most its full size.
import * as d3 from "d3";

/** A cover's full size, which it scales down from when the row is narrower. */
const WIDTH = 92;
const HEIGHT = 138;
const GAP = 10;
const LIFT = 10;

// Cover colours that keep white text readable, in light and dark alike.
const COLOURS = ["#3557c4", "#b4462b", "#0f7a6c", "#8a3ea8", "#a8650a", "#4f7a1c", "#9c2f5a"];

// A number from `text`, the same at every render, so a book always gets the same cover.
function hash(text) {
  let h = 2166136261;
  for (const c of text) h = Math.imul(h ^ c.codePointAt(0), 16777619);
  return h >>> 0;
}

// The title in lines of at most `width` characters, at most three, the last cut short.
function lines(title, width = 11) {
  const out = [];
  for (const word of title.split(" ")) {
    const last = out.at(-1);
    if (last !== undefined && last.length + 1 + word.length <= width) out[out.length - 1] = `${last} ${word}`;
    else out.push(word);
  }
  return out.length > 3 ? [...out.slice(0, 2), `${out[2]}…`] : out;
}

// The cover's pattern, one of three by the title: stripes, rings or a band. Drawn past the cover's edges, it is clipped
// to the cover by `clip`, so it neither shows nor catches clicks beyond it, over the next cover.
function pattern(cover, seed, clip) {
  const ink = cover.append("g").attr("clip-path", `url(#${clip})`).attr("stroke", "#fff").attr("fill", "none").attr("opacity", 0.18);
  const kind = seed % 3;
  if (kind === 0) {
    for (let x = -HEIGHT; x < WIDTH; x += 12) ink.append("line").attr("x1", x).attr("y1", HEIGHT).attr("x2", x + HEIGHT).attr("y2", 0).attr("stroke-width", 4);
  } else if (kind === 1) {
    for (let r = 10; r < 70; r += 11) ink.append("circle").attr("cx", WIDTH * 0.75).attr("cy", HEIGHT * 0.78).attr("r", r).attr("stroke-width", 2.5);
  } else {
    ink.append("rect").attr("y", HEIGHT * 0.62).attr("width", WIDTH).attr("height", 18).attr("fill", "#fff").attr("stroke", "none");
  }
}

/** The title's size, which a title too wide for its cover shrinks from, and how far it may. */
const TITLE = 14;
const SMALLEST = 9;
/** The room for a title's line: the cover, less the margin at its start and a little at its end. */
const ROOM = WIDTH - 14 - 8;

// Shrinks the title until its widest line fits the cover: character counts can't know a font's widths. A title not laid
// out yet, in a cover not yet in the page, measures nothing and stays as it is.
function fit(title) {
  const widest = Math.max(0, ...title.selectAll("tspan").nodes().map((line) => line.getComputedTextLength()));
  if (widest > ROOM) title.attr("font-size", Math.max(SMALLEST, (TITLE * ROOM) / widest));
}

// What a book is to the club, said under its cover and to a screen reader.
const labelOf = (book) => (book.status === "next" ? "Next" : book.status === "reading" ? "This month" : "");

// Draws `book`'s cover in its new `button`: the cover, its title and author, its ring, hidden until picked, its label.
function drawCover(button, book) {
  const seed = hash(book.title);
  const next = book.status === "next";
  const label = labelOf(book);
  button
    .attr("type", "button")
    .attr("role", "radio")
    .attr("aria-label", `${book.title}, ${book.author}${label ? `, ${label.toLowerCase()}` : ""}`)
    .style("flex", "1 1 0")
    .style("min-width", "0")
    .style("max-width", `${WIDTH}px`)
    .style("padding", "0")
    .style("border", "0")
    .style("background", "none")
    .style("cursor", "pointer")
    .style("transition", "transform 160ms ease-out");
  // As wide as its button, the drawing scaled with it.
  const svg = button.append("svg").attr("viewBox", `0 0 ${WIDTH} ${HEIGHT + 18}`).attr("aria-hidden", "true").style("display", "block").style("width", "100%").style("height", "auto").style("overflow", "visible");
  const colour = COLOURS[seed % COLOURS.length];
  const cover = svg.append("g");
  if (next) {
    // Not on the shelf yet: an outline, as a book on order.
    cover.append("rect").attr("x", 1).attr("y", 1).attr("width", WIDTH - 2).attr("height", HEIGHT - 2).attr("rx", 4).attr("fill", "none").attr("stroke", colour).attr("stroke-width", 2).attr("stroke-dasharray", "6 5");
  } else {
    cover.append("rect").attr("width", WIDTH).attr("height", HEIGHT).attr("rx", 4).attr("fill", colour);
    const clip = `shelf-cover-${String(seed)}`;
    svg.append("defs").append("clipPath").attr("id", clip).append("rect").attr("width", WIDTH).attr("height", HEIGHT).attr("rx", 4);
    pattern(cover, seed >>> 4, clip);
    // The spine's shadow, at the left edge.
    cover.append("rect").attr("width", 7).attr("height", HEIGHT).attr("fill", "#000").attr("opacity", 0.16);
  }
  const ink = next ? colour : "#fff";
  const title = cover.append("text").attr("x", 14).attr("y", 26).attr("fill", ink).attr("font-family", '"Iowan Old Style", Charter, Georgia, serif').attr("font-size", TITLE).attr("font-weight", 600);
  for (const [line, text] of lines(book.title).entries()) title.append("tspan").attr("x", 14).attr("dy", line === 0 ? 0 : 17).text(text);
  fit(title);
  cover.append("text").attr("x", 14).attr("y", HEIGHT - 14).attr("fill", ink).attr("opacity", 0.85).attr("font-family", "system-ui, sans-serif").attr("font-size", 10).text(book.author);
  cover.append("rect").attr("class", "ring").attr("opacity", 0).attr("x", -4).attr("y", -4).attr("width", WIDTH + 8).attr("height", HEIGHT + 8).attr("rx", 7).attr("fill", "none").attr("stroke", "currentColor").attr("stroke-width", 2);
  if (label) svg.append("text").attr("x", WIDTH / 2).attr("y", HEIGHT + 15).attr("text-anchor", "middle").attr("fill", "currentColor").attr("opacity", 0.7).attr("font-family", "system-ui, sans-serif").attr("font-size", 11).text(label);
}

// A book's cover is drawn once and kept while the book stays as it is: a pick only lifts and rings another, so the
// lift eases and the pointer and the focus stay on their cover.
const keyOf = (book) => `${book.title}\u0000${book.author}\u0000${book.status}`;

export default function mount(target, { params, set, fallback }) {
  const shelf = d3.select(target).append("div").attr("role", "radiogroup").attr("aria-label", fallback.text || "The club's books").style("display", "flex").style("gap", `${GAP}px`).style("padding-top", `${LIFT}px`);
  /** The books' titles, in order, as the last render gave them: the arrow keys go through them. */
  let titles = [];

  const choose = (title) => set("picked", title);

  function move(event, book) {
    const step = event.key === "ArrowRight" || event.key === "ArrowDown" ? 1 : event.key === "ArrowLeft" || event.key === "ArrowUp" ? -1 : 0;
    if (step === 0) return;
    event.preventDefault();
    const index = titles.indexOf(book.title);
    choose(titles[(index + step + titles.length) % titles.length]);
  }

  function draw({ books = [], picked }) {
    titles = books.map((book) => book.title);
    const current = picked ?? titles[0];
    shelf
      .selectAll("button")
      .data(books, keyOf)
      .join((enter) =>
        enter
          .append("button")
          .each(function (book) {
            drawCover(d3.select(this), book);
          })
          .on("click", (_event, book) => choose(book.title))
          .on("keydown", (event, book) => move(event, book)),
      )
      .order()
      .attr("aria-checked", (book) => String(book.title === current))
      .attr("tabindex", (book) => (book.title === current ? 0 : -1))
      .style("transform", (book) => `translateY(${book.title === current ? -LIFT : 0}px)`)
      .each(function (book) {
        d3.select(this).select(".ring").attr("opacity", book.title === current ? 1 : 0);
      });
  }

  draw(params);
  return {
    update: (next) => {
      const focused = target.contains(target.ownerDocument.activeElement);
      draw(next);
      // With the keyboard, the focus follows the pick, as in any group of radio buttons.
      if (focused) shelf.select("[aria-checked=true]").node()?.focus();
    },
    destroy: () => shelf.remove(),
  };
}
