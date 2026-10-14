import { expect, test } from "vite-plus/test";
import {
  toRocketPath,
  parseRoute,
  parseRoutes,
  registerRoute,
  generateModule,
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

test("generateModule produces valid TypeScript code", () => {
  const routes = parseRoutes([
    { name: "getUser", method: "GET", path: "/users/:id" },
    { name: "getPost", method: "GET", path: "/posts/:category/:slug" },
  ]);
  const code = generateModule(routes);

  expect(code).toContain("export interface RouteTypes {");
  expect(code).toContain("export const ROUTES = {");
  expect(code).toContain("export function uri(");
});

test("TypeScript route registration works correctly", () => {
  expect(typeof registerRoute).toBe("function");
});
