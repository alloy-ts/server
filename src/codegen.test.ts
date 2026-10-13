import { expect, test } from "vite-plus/test";
import {
  toRocketPath,
  parseRoute,
  generateModule,
} from "../dist/index.js";

test("toRocketPath converts Express/Bun patterns to Rocket syntax", () => {
  expect(toRocketPath("/users/:id")).toBe("/users/<id>");
  expect(toRocketPath("/api/:version/users/:id/*")).toBe("/api/<version>/users/<id>/<_..>");
  expect(toRocketPath("/static/*")).toBe("/static/<_..>");
  expect(toRocketPath("/plain")).toBe("/plain");
});

test("parseRoute extracts path and query dynamic parameters", () => {
  const route = parseRoute({
    method: "GET",
    path: "/posts/:slug?<query>&<rest..>",
  });
  expect(route.method).toBe("GET");
  expect(route.rocketPath).toBe("/posts/<slug>");

  expect(route.params.length).toBe(1);
  expect(route.params[0]?.name).toBe("slug");

  expect(route.query.length).toBe(2);
  expect(route.query[0]?.name).toBe("query");
  expect(route.query[1]?.name).toBe("rest");
});

test("generateModule produces valid TypeScript helper code", () => {
  const code = generateModule([
    parseRoute({ method: "GET", path: "/users/:id", name: "getUser" }),
    parseRoute({ method: "GET", path: "/posts/:category/:slug", name: "getPost" }),
  ]);

  expect(code).toContain("export const ROUTES = {");
  expect(code).toContain("export function uri(");
  expect(code).toContain("async getUser(");
  expect(code).toContain("async getPost(");
});
