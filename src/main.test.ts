import { expect, test } from "vite-plus/test";
import {
  main,
  Alloy,
  App,
  Response,
  response,
  Config,
  get,
  post,
  route,
  catchCatcher,
  routes,
  catchers,
  uri,
  parseRoutePattern,
  buildRouteUri,
  parseCatcherCode,
  generateRouteManifest,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Alloy object has serve, file, Response, Config, App, response", () => {
  expect(typeof Alloy.serve).toBe("function");
  expect(typeof Alloy.file).toBe("function");
  expect(Alloy.Response).toBe(Response);
  expect(Alloy.Config).toBe(Config);
  expect(Alloy.App).toBe(App);
  expect(Alloy.response).toBe(response);
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

  const resp2 = response(Buffer.from("Hello Buffer"), { status: 201 });
  expect(resp2.status).toBe(201);
});

test("App class methods and server launching", () => {
  const app = new App({ port: 0 });

  app.get("/", async () => response("Hello, world!"));
  app.post("/submit", async () => response("Submitted", { status: 201 }));

  const server = app.launch();
  expect(server.url).toContain("http://");
  expect(typeof server.port).toBe("number");

  app.stop();
});

test("Alloy.file returns StaticFile marker", () => {
  const f = Alloy.file("./favicon.ico");
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

test("Codegen - uri builder (buildRouteUri)", () => {
  const u1 = uri("/users/:id", { id: 123 });
  expect(u1).toBe("/users/123");

  const u2 = uri("/posts/<category>/<id>", { category: "tech", id: 456 });
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

test("Codegen - route registration and decorators", () => {
  function getUsersHandler() {
    return Response.json([]);
  }

  function createUserHandler() {
    return Response.json({ created: true });
  }

  function updateUserHandler() {
    return Response.json({ updated: true });
  }

  function notFoundHandler() {
    return new Response("Not found", { status: 404 });
  }

  get("/users")(getUsersHandler);
  post("/users")(createUserHandler);
  route("PUT", "/users/:id")(updateUserHandler);
  catchCatcher(404)(notFoundHandler);

  const allRoutes = routes();
  expect(allRoutes.some((r) => r.method === "GET" && r.path === "/users")).toBe(true);
  expect(allRoutes.some((r) => r.method === "POST" && r.path === "/users")).toBe(true);
  expect(allRoutes.some((r) => r.method === "PUT" && r.path === "/users/:id")).toBe(true);

  const allCatchers = catchers();
  expect(allCatchers.some((c) => c.code === 404)).toBe(true);
});
