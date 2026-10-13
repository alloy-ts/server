import { Rocket, responseJson } from "../dist/index.js";

// Demonstrating custom Rocket catchers via low-level Rocket NAPI class
const rocket = new Rocket({
  port: 8000,
  address: "127.0.0.1",
});

// Register a 404 catcher handler
rocket.register("/", 404, (_req: any) => {
  return responseJson(
    {
      error: "Not Found",
      message: "The requested resource could not be located.",
    },
    { status: 404 },
  );
});

// Register a 500 internal server error catcher
rocket.register("/", 500, (_req: any) => {
  return responseJson(
    {
      error: "Internal Server Error",
      message: "An unexpected error occurred.",
    },
    { status: 500 },
  );
});

// Mount a route that triggers 404/forwarding
rocket.mount("/", "GET", "/hello", async () => {
  return {
    status: 200,
    headers: { "Content-Type": "text/plain" },
    body: Array.from(Buffer.from("Hello from Rocket Catcher Example!")),
  };
});

console.log("Rocket instance configured with custom 404 and 500 catchers at http://127.0.0.1:8000");

if (process.env.NODE_ENV === "test") {
  // Configured for dry run
}
