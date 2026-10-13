import { expect, test } from "vite-plus/test";
import { parseRoute, toRocketPath, generateClient, generateModule, Alloy } from "../dist/index.js";

test("toRocketPath converts Express/Bun patterns to Rocket syntax", () => {
  expect(toRocketPath("/users/:id")).toBe("/users/<id>");
  expect(toRocketPath("/api/:version/users/:id/*")).toBe("/api/<version>/users/<id>/<_..>");
  expect(toRocketPath("/static/*")).toBe("/static/<_..>");
  expect(toRocketPath("/plain")).toBe("/plain");
});

test("parseRoute extracts path and query dynamic parameters", () => {
  const meta = parseRoute({
    method: "GET",
    path: "/posts/:slug?<query>&<rest..>",
  });
  expect(meta.method).toBe("GET");
  expect(meta.rocketPath).toBe("/posts/<slug>");

  const pathParams = meta.params.filter((p) => p.kind === "Path");
  expect(pathParams.length).toBe(1);
  expect(pathParams[0]?.name).toBe("slug");

  const queryParams = meta.query.filter((p) => p.kind === "Query");
  expect(queryParams.length).toBe(2);
  expect(queryParams[0]?.name).toBe("query");
  expect(queryParams[1]?.name).toBe("rest");
});

test("generateClient produces valid TypeScript client code", () => {
  const code = generateClient([
    parseRoute({ method: "GET", path: "/users/:id", name: "getUser" }),
    parseRoute({ method: "GET", path: "/posts/:category/:slug", name: "getPost" }),
  ]);

  expect(code).toContain("export class RocketClient {");
  expect(code).toContain("async getUser(");
  expect(code).toContain("async getPost(");
});

test("generateModule and Alloy.serve exports work correctly", () => {
  expect(typeof Alloy.serve).toBe("function");

  const modCode = generateModule([
    parseRoute({ method: "GET", path: "/api/items/:id", name: "item" }),
  ]);
  expect(modCode).toContain("export const ROUTES = {");
  expect(modCode).toContain("async item(");
});
