import { expect, test } from "vite-plus/test";
import { Alloy, Response } from "./main.ts";

test("Alloy.serve creates a Rocket server with routes and option configurations", () => {
  const server = Alloy.serve(
    {
      "/api/status": new Response("OK"),
      "/users/:id": (req: any) => new Response(`Hello User ${req.params?.id}!`),
      "/api/posts": {
        GET: () => new Response("List posts"),
        POST: () => Response.json({ created: true }),
      },
      "/blog/hello": Response.redirect("/blog/hello/world"),
      "/favicon.ico": Alloy.file("./package.json"),
    },
    (req: any) => new Response(`Not Found: ${req.path}`, { status: 404 }),
    { port: 0, hostname: "127.0.0.1" },
  );

  expect(server).toBeDefined();
  expect(typeof server.port).toBe("number");
  expect(server.port).toBeGreaterThan(0);
  expect(typeof server.url).toBe("string");
  expect(server.url).toContain("http://127.0.0.1:");

  expect(() => server.stop()).not.toThrow();
});
