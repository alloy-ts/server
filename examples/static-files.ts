/**
 * Example: Static File Serving
 *
 * Demonstrates lazily serving static files using `Alloy.file(path)`.
 */

import { Alloy, Response } from "../dist/main.mjs";

const server = Alloy.serve({
  routes: {
    "/": new Response("Welcome! Try requesting /favicon.ico or /ReadMe.md"),
    "/favicon.ico": Alloy.file("./favicon.ico"),
    "/readme": Alloy.file("./ReadMe.md"),
  },
});

console.log(`Static file server running at ${server.url}`);

setTimeout(() => {
  server.stop();
}, 2000);
