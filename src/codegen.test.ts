import { expect, test } from "vite-plus/test";
import {
  toRocketPath,
  parseRoute,
  parseRoutes,
  generateModule,
} from "../dist/index.js";

test("toRocketPath converts Express/Bun patterns to Rocket syntax", () => {
  expect(toRocketPath("/users/:id")).toBe("/users/<id>");
  expect(toRocketPath("/api/:version/users/:id/*")).toBe("/api/<version>/users/<id>/<_..>");
  expect(toRocketPath("/static/*")).toBe("/static/<_..>");
  expect(toRocketPath("/plain")).toBe("/plain");
});

test("parseRoute extracts path and query dynamic parameters", () => {
  const routeDef = parseRoute({ method: "GET", path: "/posts/:slug?<query>&<rest..>" });
  expect(routeDef.method).toBe("GET");
  expect(routeDef.rocketPath).toBe("/posts/<slug>");

  expect(routeDef.params.length).toBe(1);
  expect(routeDef.params[0]?.name).toBe("slug");

  expect(routeDef.query.length).toBe(2);
  expect(routeDef.query[0]?.name).toBe("query");
  expect(routeDef.query[1]?.name).toBe("rest");
});

test("generateModule produces valid TypeScript helper code", () => {
  const routes = parseRoutes([
    { name: "getUser", method: "GET", path: "/users/:id" },
    { name: "getPost", method: "GET", path: "/posts/:category/:slug" },
  ]);

  const code = generateModule(routes);

  expect(code).toContain("export const ROUTES = {");
  expect(code).toContain("export function uri(");
  expect(code).toContain("export class RocketClient {");
});
