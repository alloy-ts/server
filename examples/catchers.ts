import { Catcher, Response, serve } from "../index.js";

// Register catchers for HTTP status codes (e.g. 404, 500)
const notFoundCatcher = new Catcher(404, "not_found_handler");
console.log(`Catcher registered for code: ${notFoundCatcher.code}, name: ${notFoundCatcher.name}`);

// Map base prefix
notFoundCatcher.mapBase("/api");
console.log(`Mapped base: ${notFoundCatcher.base}`);

// Launch server with fallback error handling
const server = serve({
  port: 8080,
  routes: {
    "/api/good": new Response("Success!"),
    "/api/fail": () => {
      return new Response("Forbidden", { status: 403 });
    },
  },
  fetch(req) {
    return Response.json(
      {
        error: "Not Found",
        code: 404,
        path: req.path,
        catcher: notFoundCatcher.name,
      },
      { status: 404 }
    );
  },
});

console.log(`Server running at ${server.url}`);
server.stop();
