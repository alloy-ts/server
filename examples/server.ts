import {
  serve,
  response,
  responseJson,
  responseRedirect,
  file,
  type RequestInput,
} from "../dist/index.js";

const server = serve(
  {
    "/": () => response("Welcome to Rocket TypeScript Server!"),
    "/api/status": responseJson({ status: "ok", timestamp: new Date().toISOString() }),
    "/users/:id": (req: RequestInput) => response(`Hello User ${req.params?.["id"]}`),
    "/api/posts": {
      GET: () => responseJson([{ id: 1, title: "First Post" }]),
      POST: async (req: RequestInput) => responseJson({ created: true, received: req.body }),
    },
    "/docs": () => responseRedirect("/"),
    "/favicon.ico": file("./favicon.ico"),
  },
  (req: RequestInput) => responseJson({ error: "Not Found", path: req.path }, { status: 404 }),
  {
    port: 3000,
    hostname: "127.0.0.1",
  },
);

console.log(`Server listening at ${server.url}`);
