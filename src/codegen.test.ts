import { expect, test } from "vite-plus/test";
import {
  Catch,
  Delete,
  FromForm,
  Get,
  Head,
  Options,
  Patch,
  Post,
  Put,
  Responder,
  Response,
  Route,
  catchers,
  generateTsRouteManifest,
  generateUri,
  parseCatchAttribute,
  parseRouteAttribute,
  routes,
  uri,
} from "./main.ts";

test("parseRouteAttribute parses Rocket route attribute syntax and grammar", () => {
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

test("parseCatchAttribute parses status code and default catchers", () => {
  const status404 = parseCatchAttribute("404");
  expect(status404.code).toBe(404);
  expect(status404.isDefault).toBe(false);

  const defaultCatcher = parseCatchAttribute("default");
  expect(defaultCatcher.code).toBeUndefined();
  expect(defaultCatcher.isDefault).toBe(true);
});

test("generateUri and uri helper interpolate path and query parameters with percent encoding", () => {
  const formattedUri1 = uri("/person/<name>?<age>", { name: "Mike Smith", age: "28" });
  expect(formattedUri1).toBe("/person/Mike%20Smith?age=28");

  const formattedUri2 = generateUri("/ignore/<_>/<other>", { other: "cat&dog" });
  expect(formattedUri2).toBe("/ignore/<_>/cat%26dog");
});

test("generateTsRouteManifest generates type-safe TypeScript route manifest code", () => {
  const manifest = generateTsRouteManifest([
    { name: "getUser", method: "GET", uri: "/users/<id>" },
    { name: "createPost", method: "POST", uri: "/posts?<tag>" },
  ]);
  expect(manifest).toContain('name: "getUser"');
  expect(manifest).toContain("getUser(params: { id: string }): string");
  expect(manifest).toContain("createPost(params: { tag?: string }): string");
});

test("Rocket decorators create and collect route definitions and catchers", () => {
  const getRoute = Get("/api/test")(() => new Response("OK"));
  const postRoute = Post("/api/test")(() => new Response("Created"));
  const putRoute = Put("/api/test")(() => new Response("Updated"));
  const deleteRoute = Delete("/api/test")(() => new Response("Deleted"));
  const patchRoute = Patch("/api/test")(() => new Response("Patched"));
  const headRoute = Head("/api/test")(() => new Response("Head"));
  const optionsRoute = Options("/api/test")(() => new Response("Options"));
  const genericRoute = Route("GET", "/api/generic")(() => new Response("Generic"));

  const table = routes(
    getRoute,
    postRoute,
    putRoute,
    deleteRoute,
    patchRoute,
    headRoute,
    optionsRoute,
    genericRoute,
  );
  expect(table["/api/test"]?.["GET"]).toBeDefined();
  expect(table["/api/test"]?.["POST"]).toBeDefined();
  expect(table["/api/test"]?.["PUT"]).toBeDefined();
  expect(table["/api/test"]?.["DELETE"]).toBeDefined();
  expect(table["/api/generic"]?.["GET"]).toBeDefined();

  const catcher = Catch(500)(() => new Response("Server Error", { status: 500 }));
  const catcherList = catchers(catcher);
  expect(catcherList.length).toBe(1);
  expect(catcherList[0]?.code).toBe(500);
});

test("FromForm helper parses form data strings and objects", () => {
  const parsed = FromForm.parse<{ title: string; category: string }>("title=Rocket&category=web");
  expect(parsed).toEqual({ title: "Rocket", category: "web" });

  const direct = FromForm.parse({ a: "1" });
  expect(direct).toEqual({ a: "1" });
});

test("Responder helper formats string, JSON, and custom response values", () => {
  const textResp = Responder.format("Hello Plain");
  expect(textResp.status).toBe(200);
  expect(textResp.headers["Content-Type"]).toBe("text/plain");

  const jsonResp = Responder.format({ active: true });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");
});
