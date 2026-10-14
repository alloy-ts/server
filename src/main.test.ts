import { expect, test } from "vite-plus/test";
import {
  main,
  file,
  Response,
  response,
  Config,
  parseRoutePattern,
  buildRouteUri,
  parseCatcherCode,
  generateRouteManifest,
} from "../index.js";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Response static constructors and instance methods", () => {
  const textResp = new Response("hello text", { status: 200 });
  expect(textResp.status).toBe(200);

  const jsonResp = Response.json({ success: true, count: 42 }, { status: 201 });
  expect(jsonResp.status).toBe(201);
  const headers = jsonResp.headers;
  expect(
    headers.some(([k, v]) => k.toLowerCase() === "content-type" && v === "application/json"),
  ).toBe(true);

  const redirResp = Response.redirect("/new-location", 302);
  expect(redirResp.status).toBe(302);
  expect(redirResp.headers.some(([k, v]) => k === "Location" && v === "/new-location")).toBe(true);
});

test("response helper function", () => {
  const resp1 = response("Hello, world!");
  expect(resp1.status).toBe(200);

  const resp2 = response("Hello String", { status: 201 });
  expect(resp2.status).toBe(201);
});

test("file returns StaticFile marker", () => {
  const f = file("./favicon.ico");
  expect(f.path).toBe("./favicon.ico");
});

test("Rocket Config getters, setters, and defaults", () => {
  const debugCfg = Config.debugDefault();
  expect(debugCfg.port).toBe(8000);
  expect(debugCfg.logLevel).toBe("normal");

  const releaseCfg = Config.releaseDefault();
  expect(releaseCfg.port).toBe(8000);

  const customCfg = new Config();
  customCfg.port = 9090;
  expect(customCfg.port).toBe(9090);

  customCfg.address = "127.0.0.1";
  expect(customCfg.address).toBe("127.0.0.1");
});

test("Codegen - parseRoutePattern parses express and rocket parameters", () => {
  const info1 = parseRoutePattern("/users/:id");
  expect(info1.rawPattern).toBe("/users/:id");
  expect(info1.rocketPattern).toBe("/users/<id>");
  expect(info1.pathParams).toEqual(["id"]);
  expect(info1.isWildcard).toBe(false);

  const info2 = parseRoutePattern("/api/*");
  expect(info2.rocketPattern).toBe("/api/<_..>");
  expect(info2.isWildcard).toBe(true);
});

test("Codegen - buildRouteUri", () => {
  const u1 = buildRouteUri("/users/:id", { id: "123" });
  expect(u1).toBe("/users/123");

  const u2 = buildRouteUri("/posts/<category>/<id>", { category: "tech", id: "456" });
  expect(u2).toBe("/posts/tech/456");

  const u3 = buildRouteUri("/search/:q", { q: "hello world" });
  expect(u3).toBe("/search/hello%20world");
});

test("Codegen - parseCatcherCode handles 404, default, and strings", () => {
  expect(parseCatcherCode("404")).toBe(404);
  expect(parseCatcherCode("default")).toBeNull();
  expect(parseCatcherCode("_")).toBeNull();
  expect(parseCatcherCode("500")).toBe(500);
});

test("Codegen - generateRouteManifest produces info array", () => {
  const manifest = generateRouteManifest(["/api/v1/posts", "/users/:id"]);
  expect(manifest.length).toBe(2);
  expect(manifest[0]?.rawPattern).toBe("/api/v1/posts");
  expect(manifest[1]?.pathParams).toEqual(["id"]);
});
