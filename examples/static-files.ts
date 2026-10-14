import { Alloy, Bun, response } from "../dist/index.js";

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/": response("Welcome to Static File Server Example"),
    "/favicon.ico": Bun.file("./favicon.ico"),
    "/static/*": (req: any) => {
      return Bun.file(`./public/${req.params._}`);
    },
  },
});

console.log(`Static File Server running at ${server.url}`);
