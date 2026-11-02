export default function mount(target, { params }) {
  target.textContent = `${params.title} (1.0.0)`;
}
