import { Alloy, response } from "../dist/index.js";

const server = Alloy.serve({
  routes: {
    "/": async () => response("Hello, world!"),
  },
});

console.log(`Server running at ${server.url}`);
