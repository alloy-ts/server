import { serve, response, responseJson, responseRedirect, file } from "../dist/index.js";

const server = serve({
  routes: {
    "/api/status": response("OK"),

    "/users/:id": (req: any) => {
      return response(`Hello User ${req.params?.id}!`);
    },

    "/api/posts": {
      GET: () => response("List posts"),
      POST: async () => responseJson({ created: true }),
    },

    "/blog/hello": responseRedirect("/blog/hello/world"),

    "/favicon.ico": file("./favicon.ico"),
  },

  fetch(_req: any) {
    return response("Not Found", { status: 404 });
  },
});

console.log(`Server running at ${server.url}`);
