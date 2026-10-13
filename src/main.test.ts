import { expect, test } from "vite-plus/test";
import { file, Response, serve } from "./main.ts";

test("Response constructors and methods work as expected", () => {
  const resp = new Response("hello world");
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ key: "value" }, { status: 201 });
  expect(jsonResp.status).toBe(201);

  const redirectResp = Response.redirect("http://example.com", 301);
  expect(redirectResp.status).toBe(301);

  const fileMarker = file("./test.txt");
  expect(fileMarker.path).toBe("./test.txt");
});

test("serve creates a server and handles routes table and HTTP requests", async () => {
  const server = serve({
    port: 3001,
    routes: {
      "/api/status": new Response("OK"),
      "/users/:id": async (req: unknown) => {
        console.log("DEBUG /users/:id req:", req);
        return new Response(`Hello User ${(req as any)?.params?.id}!`);
      },
    },
  });

  try {
    const res2 = await fetch(`${server.url}/users/42`);
    console.log("res2 status:", res2.status);
    const text2 = await res2.text();
    console.log("text2:", text2);
  } finally {
    server.stop();
  }
});
