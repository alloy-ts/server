import { serve, Response, file } from "../dist/index.js";

const server = serve(
  {
    // Static routes
    "/api/status": new Response("OK"),

    // Dynamic routes
    "/users/:id": (req: any) => {
      return new Response(`Hello User ${req.params?.id}!`);
    },

    // Per-HTTP method handlers
    "/api/posts": {
      GET: () => new Response("List posts"),
      POST: async (req: any) => {
        const body = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        return Response.json({ created: true, ...body });
      },
    },

    // Redirect from /blog/hello to /blog/hello/world
    "/blog/hello": Response.redirect("/blog/hello/world"),

    // Serve a file
    "/favicon.ico": file("./package.json"),
  },

  // (optional) fallback for unmatched routes:
  (req: any) => {
    return new Response(`Not Found: ${req.path}`, { status: 404 });
  },
);

console.log(`Server running at ${server.url}`);
