export default function Board({ by, compmd }) {
  const lanes = Object.groupBy(compmd.slots.default, (instance) => String(instance.value[by]));
  return (
    <div className="board">
      {Object.entries(lanes).map(([lane, instances]) => (
        <section key={lane} aria-label={lane}>
          {instances.map((instance, i) => <SlotItems key={i} items={instance.items} />)}
        </section>
      ))}
    </div>
  );
}

function SlotItems({ items }) {
  return <div ref={(el) => el && el.replaceChildren(...items.map((item) => item.element))} />;
}
