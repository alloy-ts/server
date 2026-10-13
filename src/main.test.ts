import { expect, test } from "vite-plus/test";
import { main, Alloy, Response, Config, buildUri, generateRoutesManifest } from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Response class constructs and serializes correctly", () => {
  const resp = new Response("hello world", { status: 200 });
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ key: "value" }, { status: 201 });
  expect(jsonResp.status).toBe(201);

  const redirectResp = Response.redirect("https://rocket.rs", 302);
  expect(redirectResp.status).toBe(302);
});

test("buildUri replaces parameters correctly", () => {
  const uri = buildUri("/users/:id/posts/<postId>", {
    id: "42",
    postId: "100",
  });
  expect(uri).toBe("/users/42/posts/100");
});

test("generateRoutesManifest generates valid TypeScript code", () => {
  const manifest = generateRoutesManifest([
    {
      method: "GET",
      uri: "/api/status",
      name: "status",
      rank: 1,
      format: "json",
    },
  ]);
  expect(manifest).toContain("export const routesManifest = [");
  expect(manifest).toContain('"GET"');
  expect(manifest).toContain('"/api/status"');
});

test("Config exposes default options", () => {
  const config = Config.debugDefault();
  expect(config.port).toBeDefined();
  expect(config.address).toBeDefined();
});

test("Alloy.serve launches a Rocket web server and responds to HTTP requests", async () => {
  const port = 3000 + Math.floor(Math.random() * 5000);
  const server = Alloy.serve({
    port,
    hostname: "127.0.0.1",
    routes: {
      "/api/status": new Response("OK"),
      "/users/:id": (req: any) => {
        const id = req?.params?.id || "123";
        return new Response(`Hello User ${id}!`);
      },
      "/api/json": () => {
        return Response.json({ status: "success" });
      },
    },
  });

  expect(server.url).toBe(`http://127.0.0.1:${port}`);
  expect(server.port).toBe(port);

  // Give rocket a brief moment to liftoff
  await new Promise((r) => setTimeout(r, 200));

  try {
    const res1 = await fetch(`${server.url}/api/status`);
    expect(res1.status).toBe(200);
    const text1 = await res1.text();
    expect(text1).toBe("OK");

    const res2 = await fetch(`${server.url}/users/123`);
    expect(res2.status).toBe(200);
    const text2 = await res2.text();
    expect(text2).toBe("Hello User 123!");

    const res3 = await fetch(`${server.url}/api/json`);
    expect(res3.status).toBe(200);
    const data = await res3.json();
    expect(data).toEqual({ status: "success" });
  } finally {
    server.stop();
  }
});
