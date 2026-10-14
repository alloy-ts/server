import { expect, test } from "vite-plus/test";
import {
  buildRouteUri,
  Config,
  file,
  generateTsRouteTypes,
  parseRoutePattern,
  registerRoute,
  Response,
  Route,
  serve,
  uri,
} from "../dist/index.js";

test("Response static constructors and getters", () => {
  const resp = new Response("hello world", {
    status: 200,
    headers: [{ name: "x-custom", value: "value" }],
  });
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ status: "ok" });
  expect(jsonResp.status).toBe(200);

  const redirectResp = Response.redirect("/login", 301);
  expect(redirectResp.status).toBe(301);
});

test("Config default profiles and parameters", () => {
  const cfg = new Config();
  expect(cfg.port).toBeGreaterThan(0);
  expect(Config.parameters()).toContain("port");
});

test("Route class", () => {
  const route = new Route("GET", "/test/<id>", "test_route", 1, "json");
  expect(route.method).toBe("GET");
  expect(route.uri).toBe("/test/<id>");
  expect(route.name).toBe("test_route");
  expect(route.rank).toBe(1);
  expect(route.format).toBe("json");

  const rankedRoute = Route.ranked(2, "POST", "/posts");
  expect(rankedRoute.method).toBe("POST");
  expect(rankedRoute.uri).toBe("/posts");
  expect(rankedRoute.rank).toBe(2);
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

  const tsDefs = generateTsRouteTypes(["/api/items/<id>"]);
  expect(tsDefs).toContain("export interface ApiRoutes");
  expect(tsDefs).toContain("id: string;");
});

test("Route registration", () => {
  expect(() => {
    registerRoute("GET", "/api/health", async () => new Response("OK"));
  }).not.toThrow();
});

test("serve creates server instance", () => {
  const server = serve(
    {
      "/api/status": new Response("OK"),
    },
    null,
    { port: 0 },
  );

  expect(server.url).toContain("http://");
  expect(server.port).toBeGreaterThanOrEqual(0);

  server.stop();
});

test("file helper creates static file definition", () => {
  const f = file("./package.json");
  expect(f.path).toBe("./package.json");
});
