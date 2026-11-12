// A source component of the pack: the bundle compiles it ahead, and the frame loads `Total.jsx.js` by its URL.
export default function Total({ values }) {
  const total = values.reduce((sum, value) => sum + value, 0);
  return <p>{total} pages in all</p>;
}
