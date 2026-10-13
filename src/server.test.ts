import { expect, test } from "vite-plus/test";
import { Alloy } from "./main.ts";

test("server starts on port and handles routes and stop", async () => {
  const server = Alloy.serve({
    port: 0,
    hostname: "127.0.0.1",
    routes: {
      "/api/hello": new Alloy.Response("Hello World"),
      "/users/:id": (req: { params: Record<string, string> }) =>
        new Alloy.Response(`User ${req.params["id"]}`),
    },
    fetch: () => new Alloy.Response("Fallback 404", { status: 404 }),
  });

  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://127.0.0.1:");

  // Fetch /api/hello
  const res1 = await fetch(`${server.url}/api/hello`);
  const text1 = await res1.text();
  expect(res1.status).toBe(200);
  expect(text1).toBe("Hello World");

  // Fetch /users/123 (dynamic param)
  const res2 = await fetch(`${server.url}/users/123`);
  const text2 = await res2.text();
  expect(res2.status).toBe(200);
  expect(text2).toBe("User 123");

  // Fetch unmatched route (fallback)
  const res3 = await fetch(`${server.url}/unmatched`);
  const text3 = await res3.text();
  expect(res3.status).toBe(404);
  expect(text3).toBe("Fallback 404");

  server.stop();
});
