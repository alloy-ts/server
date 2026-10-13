import { file, response, serve } from "../src/main.ts";

const server = serve(
  {
    "/": async () => response("Welcome to the Static File Server!"),
    "/favicon.ico": file("./favicon.ico"),
    "/assets/logo.png": file("./assets/logo.png"),
  },
  async (_req) => response("File Not Found", { status: 404 }),
  { port: 8080 },
);

console.log(`File Server listening at ${server.url}`);
