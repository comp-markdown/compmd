// Serves the built site, `dist/`, as GitHub Pages will: `pnpm preview [port]`, port 4173 by default.
import { createServer } from "node:http";
import { join } from "node:path";
import { serveFolder } from "../build/static.ts";

const port = Number(process.argv[2] ?? 4173);
const serve = serveFolder("/", join(import.meta.dirname, "../dist"));
createServer((request, response) => {
  serve(request, response, () => {
    response.statusCode = 404;
    response.end("Not found");
  });
}).listen(port, () => {
  console.log(`The site: http://localhost:${String(port)}/`);
});
