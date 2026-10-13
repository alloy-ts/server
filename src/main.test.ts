import { expect, test } from "vite-plus/test";
import {
  Catch,
  Config,
  FromForm,
  Get,
  Post,
  Responder,
  Response,
  catchers,
  generateTsRouteManifest,
  generateUri,
  main,
  parseCatchAttribute,
  parseRouteAttribute,
  routes,
  uri,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Response creates responses with status, headers, and body", () => {
  const resp = new Response("Hello World", { status: 200, headers: { "X-Custom": "Value" } });
  expect(resp.status).toBe(200);
  expect(resp.headers["X-Custom"]).toBe("Value");
  expect(resp.body?.toString()).toBe("Hello World");
});

test("Response.json formats json output", () => {
  const resp = Response.json({ ok: true, id: 123 });
  expect(resp.status).toBe(200);
  expect(resp.headers["Content-Type"]).toBe("application/json");
  expect(JSON.parse(resp.body?.toString() || "{}")).toEqual({ ok: true, id: 123 });
});

test("Response.redirect creates redirect response", () => {
  const resp = Response.redirect("/login", 303);
  expect(resp.status).toBe(303);
  expect(resp.headers["Location"]).toBe("/login");
});

test("Config initializes Rocket server configuration", () => {
  const config = new Config();
  expect(config.port).toBe(8000);
  expect(config.address).toBe("127.0.0.1");

  config.port = 9090;
  expect(config.port).toBe(9090);
});

test("parseRouteAttribute parses Rocket route attribute grammar", () => {
  const parsed = parseRouteAttribute(
    '"/person/<name>?<age>"',
    'rank = 1, format = "json", data = "<form>"',
  );
  expect(parsed.uri).toBe("/person/<name>?<age>");
  expect(parsed.path).toBe("/person/<name>");
  expect(parsed.query).toBe("<age>");
  expect(parsed.pathParams).toEqual(["name"]);
  expect(parsed.queryParams).toEqual(["age"]);
  expect(parsed.rank).toBe(1);
  expect(parsed.format).toBe("json");
  expect(parsed.dataParam).toBe("form");
});

test("parseCatchAttribute parses Rocket catch status or default", () => {
  const statusCatch = parseCatchAttribute("404");
  expect(statusCatch.code).toBe(404);
  expect(statusCatch.isDefault).toBe(false);

  const defaultCatch = parseCatchAttribute("default");
  expect(defaultCatch.code).toBeUndefined();
  expect(defaultCatch.isDefault).toBe(true);
});

test("generateUri and uri build URL-safe route URIs", () => {
  const uri1 = uri("/person/<name>?<age>", { name: "Mike Smith", age: "28" });
  expect(uri1).toBe("/person/Mike%20Smith?age=28");

  const uri2 = generateUri("/ignore/<_>/<other>", { other: "cat&dog" });
  expect(uri2).toBe("/ignore/<_>/cat%26dog");
});

test("Rocket route creators and route collection register routes and catchers", () => {
  const getUserRoute = Get("/users/:id")(() => new Response("User"));
  const createUserRoute = Post("/users")(() => new Response("Created"));
  const notFoundCatch = Catch(404)(() => new Response("Not Found", { status: 404 }));

  const table = routes(getUserRoute, createUserRoute);
  expect(table["/users/:id"]?.["GET"]).toBeDefined();
  expect(table["/users"]?.["POST"]).toBeDefined();

  const catcherList = catchers(notFoundCatch);
  expect(catcherList.length).toBe(1);
  expect(catcherList[0]?.code).toBe(404);
});

test("generateTsRouteManifest generates TypeScript route manifest boilerplate", () => {
  const manifest = generateTsRouteManifest([
    { name: "getUser", method: "GET", uri: "/users/<id>" },
  ]);
  expect(manifest).toContain('name: "getUser"');
  expect(manifest).toContain("getUser(params: { id: string }): string");
});

test("FromForm and Responder helpers handle form and response data", () => {
  const parsedForm = FromForm.parse<{ name: string; age: string }>("name=John&age=30");
  expect(parsedForm).toEqual({ name: "John", age: "30" });

  const responderFormatted = Responder.format("Hello World");
  expect(responderFormatted.status).toBe(200);
  expect(responderFormatted.headers["Content-Type"]).toBe("text/plain");
});
