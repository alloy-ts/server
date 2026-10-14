import { expect, test } from "vite-plus/test";
import {
  Alloy,
  App,
  buildRouteUri,
  Config,
  file,
  generateRoutesTypeDefinitions,
  parseRoutePattern,
  registerRoute,
  Response,
  response,
  uri,
} from "../dist/index.js";

test("Response static constructors and getters", () => {
  const resp = new Response("hello world", { status: 200, headers: [["x-custom", "value"]] });
  expect(resp.status).toBe(200);
  expect(resp.headers).toEqual([["x-custom", "value"]]);

  const jsonResp = Response.json({ status: "ok" });
  expect(jsonResp.status).toBe(200);

  const redirectResp = Response.redirect("/login", 301);
  expect(redirectResp.status).toBe(301);
  expect(redirectResp.headers).toEqual([["Location", "/login"]]);
});

test("response helper constructs Response with status", () => {
  const resp1 = response("Hello, world!");
  expect(resp1.status).toBe(200);

  const resp2 = response("Created", 201, [["x-type", "test"]]);
  expect(resp2.status).toBe(201);
  expect(resp2.headers).toEqual([["x-type", "test"]]);
});

test("Config default profiles and parameters", () => {
  const cfg = new Config();
  expect(cfg.port).toBeGreaterThan(0);
  expect(Config.parameters()).toContain("port");
});

test("Codegen utilities: uri and route parsing", () => {
  expect(uri("/users/:id", { id: "42" })).toBe("/users/42");

  const routeInfo = parseRoutePattern("/users/<id>/<action..>?<ref>");
  expect(routeInfo.pathParams).toEqual([
    { name: "id", isTrailing: false },
    { name: "action", isTrailing: true },
  ]);
  expect(routeInfo.queryParams).toEqual(["ref"]);

  const uriBuilt = buildRouteUri("/users/<id>?<ref>", { id: "10", ref: "test" });
  expect(uriBuilt).toBe("/users/10?ref=test");

  const tsDefs = generateRoutesTypeDefinitions(["/api/items/<id>"]);
  expect(tsDefs).toContain("export interface ApiRoutes");
  expect(tsDefs).toContain("id: string;");
});

test("Route registration", () => {
  expect(() => registerRoute("GET", "/api/health", () => new Response("OK"))).not.toThrow();
  expect(() =>
    registerRoute("POST", "/api/data", () => new Response("Created", { status: 201 })),
  ).not.toThrow();
});

test("App class methods and launch/stop lifecycle", () => {
  const app = new App();
  app.get("/hello", async () => response("Hello, world!"));
  app.post("/data", async () => response("Created", 201));

  const server = app.launch({ port: 0 });
  expect(server.url).toContain("http://");
  expect(server.port).toBeGreaterThanOrEqual(0);

  app.stop();
});

test("Alloy.serve creates server instance", () => {
  const server = Alloy.serve({
    port: 0,
    routes: {
      "/api/status": new Response("OK"),
      "/users/:id": (req: any) => new Response(`User ${req.params.id}`),
    },
    fetch() {
      return new Response("Not Found", { status: 404 });
    },
  });

  expect(server.url).toContain("http://");
  expect(server.port).toBeGreaterThanOrEqual(0);

  server.stop();
});

test("file helper creates static file definition", () => {
  const f = file("./public/index.html");
  expect(f.path).toBe("./public/index.html");
});
