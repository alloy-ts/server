import { expect, test } from "vite-plus/test";
import {
  Alloy,
  buildRouteUri,
  Config,
  file,
  generateRoutesTypeDefinitions,
  get,
  getRegisteredRoutesMeta,
  Handler,
  main,
  parseRoutePattern,
  post,
  Response,
  RouteMetadata,
  uri,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Response static constructors and getters", () => {
  const resp = new Response("hello world", { status: 200, headers: [["x-custom", "value"]] });
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ status: "ok" });
  expect(jsonResp.status).toBe(200);

  const redirectResp = Response.redirect("/login", 301);
  expect(redirectResp.status).toBe(301);
});

test("Config default profiles and parameters", () => {
  if (Config) {
    const cfg = new Config();
    expect(cfg.port).toBeGreaterThan(0);
    expect(Config.parameters()).toContain("port");
  }
});

test("RouteMetadata class", () => {
  if (RouteMetadata) {
    const route = new RouteMetadata("GET", "/test/<id>", "test_route", 1, "json");
    expect(route.method).toBe("GET");
    expect(route.uri).toBe("/test/<id>");
    expect(route.name).toBe("test_route");
    expect(route.rank).toBe(1);
    expect(route.format).toBe("json");

    const rankedRoute = RouteMetadata.ranked(2, "POST", "/posts");
    expect(rankedRoute.method).toBe("POST");
    expect(rankedRoute.uri).toBe("/posts");
    expect(rankedRoute.rank).toBe(2);
  }
});

test("Handler NAPI wrapper", () => {
  if (Handler) {
    const handler = new Handler("custom_handler");
    expect(handler.name).toBe("custom_handler");

    const succOutcome = Handler.success({ status: 200, headers: [] });
    expect(succOutcome.status).toBe(200);
    expect(succOutcome.isForward).toBe(false);

    const errOutcome = Handler.error(500);
    expect(errOutcome.status).toBe(500);

    const fwdOutcome = Handler.forward(404);
    expect(fwdOutcome.status).toBe(404);
    expect(fwdOutcome.isForward).toBe(true);
  }
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
  const f = file("./package.json");
  expect(f.path).toBe("./package.json");
});
