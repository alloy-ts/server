import { Server, responseJson } from "../dist/index.js";

// Demonstrating custom catchers via low-level Server NAPI class
const server = Server.custom({
  port: 8000,
  address: "127.0.0.1",
});

// Register a 404 catcher handler
server.register("/", 404, (_req: any) => {
  return responseJson(
    {
      error: "Not Found",
      message: "The requested resource could not be located.",
    },
    { status: 404 },
  );
});

// Register a 500 internal server error catcher
server.register("/", 500, (_req: any) => {
  return responseJson(
    {
      error: "Internal Server Error",
      message: "An unexpected error occurred.",
    },
    { status: 500 },
  );
});

// Mount a route
server.mount("/", "GET", "/hello", async () => {
  return {
    status: 200,
    headers: { "Content-Type": "text/plain" },
    body: Array.from(Buffer.from("Hello from Catcher Example!")),
  };
});

console.log("Server instance configured with custom 404 and 500 catchers at http://127.0.0.1:8000");

if (process.env.NODE_ENV === "test") {
  // Configured for dry run
}
