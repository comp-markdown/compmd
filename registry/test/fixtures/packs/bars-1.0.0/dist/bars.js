import { version } from "@kit/d3";

export default (target, { params }) => {
  target.textContent = `${params.title} (d3 ${version})`;
};
