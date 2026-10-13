import { serve, response, responseJson, responseRedirect, file } from "../dist/index.js";

// Alloy compatibility shim for Bun.serve / Alloy.serve
const Alloy = {
  serve(options: {
    routes?: Record<string, any>;
    fetch?: (req: any) => any;
    port?: number;
    hostname?: string;
    unix?: string;
  }) {
    const { routes, fetch, port, hostname, unix } = options;
    return serve(routes, fetch, { port, hostname, unix });
  },
  file(path: string) {
    return file(path);
  },
};

// Response class shim with Bun / Alloy ergonomics
class ResponseShim {
  status: number;
  headers: Record<string, string>;
  body?: number[];

  constructor(body?: string, init?: { status?: number; headers?: Record<string, string> }) {
    const res = response(body, init);
    this.status = res.status;
    this.headers = res.headers;
    this.body = res.body;
  }

  static json(data: any, init?: { status?: number; headers?: Record<string, string> }) {
    return responseJson(data, init);
  }

  static redirect(url: string, status?: number) {
    return responseRedirect(url, status);
  }
}

const Response = ResponseShim;

// Basic Setup using Alloy.serve
const server = Alloy.serve({
  port: 3000,
  routes: {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes with req.params
    "/users/:id": (req: any) => {
      return new Response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req: any) => {
        let bodyData = {};
        if (req.body) {
          try {
            bodyData = JSON.parse(Buffer.from(req.body).toString("utf-8"));
          } catch {
            // body parsing fallback
          }
        }
        return Response.json({ created: true, ...bodyData });
      },
    },

    // Wildcard route for all routes that start with "/api/" and aren't otherwise matched
    "/api/*": Response.json({ message: "Not found" }, { status: 404 }),

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file by lazily loading it into memory
    "/favicon.ico": Alloy.file("./favicon.ico"),
  },

  // Fallback for unmatched routes
  fetch(_req: any) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);

// Graceful cleanup demonstration
setTimeout(() => {
  console.log("Stopping server...");
  server.stop();
}, 1000);
