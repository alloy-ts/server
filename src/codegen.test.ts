import { expect, test } from "vite-plus/test";
import {
  generateClient,
  generateModule,
  generateTypes,
  httpMethods,
  parseRoute,
  parseRoutes,
  toRocketPath,
} from "../dist/index.js";

test("toRocketPath converts Express/Bun patterns to Rocket syntax", () => {
  expect(toRocketPath("/users/:id")).toBe("/users/<id>");
  expect(toRocketPath("/api/:version/users/:id/*")).toBe("/api/<version>/users/<id>/<_..>");
  expect(toRocketPath("/static/*")).toBe("/static/<_..>");
  expect(toRocketPath("/plain")).toBe("/plain");
});

test("parseRoute extracts path and query dynamic parameters", () => {
  const meta = parseRoute({ method: "GET", path: "/posts/:slug?<query>&<rest..>" });
  expect(meta.method).toBe("GET");
  expect(meta.rocketPath).toBe("/posts/<slug>");

  expect(meta.params.length).toBe(1);
  expect(meta.params[0]?.name).toBe("slug");

  expect(meta.query.length).toBe(2);
  expect(meta.query[0]?.name).toBe("query");
  expect(meta.query[1]?.name).toBe("rest");
});

test("generateClient produces valid TypeScript client code", () => {
  const defs = parseRoutes([
    { method: "GET", path: "/users/:id" },
    { method: "GET", path: "/posts/:category/:slug" },
  ]);
  const code = generateClient(defs);

  expect(code).toContain("export class RocketClient");
  expect(code).toContain("async getUsersById");
  expect(code).toContain("async getPostsByCategoryBySlug");
  expect(code).toContain("export function uri");
});

test("generateTypes and generateModule produce TypeScript definitions", () => {
  const defs = parseRoutes([{ method: "GET", path: "/users/:id" }]);
  const types = generateTypes(defs);
  expect(types).toContain("export interface RouteTypes");

  const mod = generateModule(defs);
  expect(mod).toContain("export class RocketClient");
});

test("httpMethods lists accepted Rocket methods", () => {
  const methods = httpMethods();
  expect(methods).toContain("GET");
  expect(methods).toContain("POST");
  expect(methods).toContain("DELETE");
});
