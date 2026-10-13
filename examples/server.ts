import { Alloy, Bun, response, responseJson, responseRedirect } from "../dist/index.js";

// Helper mirroring Web / Bun `Response` API backed by Rocket NAPI bindings
class Response {
  status: number;
  headers: Record<string, string>;
  body?: number[];

  constructor(body?: string, init?: { status?: number; headers?: Record<string, string> }) {
    const res = response(body, init);
    this.status = res.status;
    this.headers = res.headers;
    this.body = res.body;
  }

  static json(data: unknown, init?: { status?: number; headers?: Record<string, string> }) {
    return responseJson(data, init);
  }

  static redirect(url: string, status?: number) {
    return responseRedirect(url, status);
  }
}

const server = Alloy.serve({
  // `routes` table supported by Rocket NAPI bindings
  routes: {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
    "/users/:id": (req: { params: Record<string, string> }) => {
      return new Response(`Hello User ${req.params.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req: { json: () => Promise<unknown> }) => {
        const body = await req.json();
        return Response.json({ created: true, ...(body as object) });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // (optional) fallback for unmatched routes
  fetch(req: unknown) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

// Cleanly stop server and exit after demo startup
setTimeout(() => {
  server.stop();
  console.log("Server stopped successfully.");
  process.exit(0);
}, 200);
