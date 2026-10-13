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
  expect(meta.params[0].name).toBe("slug");

  expect(meta.query.length).toBe(2);
  expect(meta.query[0].name).toBe("query");
  expect(meta.query[1].name).toBe("rest");
});

test("generateModule produces valid TypeScript code", () => {
  const routes = parseRoutes([
    { method: "GET", path: "/users/:id", name: "getUser" },
    { method: "GET", path: "/posts/:category/:slug", name: "getPost" },
  ]);
  const code = generateModule(routes);

  expect(code).toContain("export const ROUTES = {");
  expect(code).toContain("export interface RouteTypes {");
  expect(code).toContain("export function uri(");
  expect(code).toContain("export class RocketClient {");
});

test("httpMethods returns supported HTTP methods", () => {
  const methods = httpMethods();
  expect(methods).toContain("GET");
  expect(methods).toContain("POST");
  expect(methods).toContain("DELETE");
});
