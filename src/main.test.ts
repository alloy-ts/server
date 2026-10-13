import { expect, test } from "vite-plus/test";
import {
  Alloy,
  buildRouteUri,
  Config,
  file,
  generateRoutesTypeDefinitions,
  get,
  getRegisteredRoutesMeta,
  parseRoutePattern,
  post,
  Response,
  uri,
} from "./main.ts";

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

test("Route registration and metadata", () => {
  get("/api/health", () => new Response("OK"));
  post("/api/data", () => new Response("Created", { status: 201 }));

  const meta = getRegisteredRoutesMeta();
  expect(meta.some((m) => m.path === "/api/health" && m.method === "GET")).toBe(true);
  expect(meta.some((m) => m.path === "/api/data" && m.method === "POST")).toBe(true);
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
