import { expect, test } from "vite-plus/test";
import { main, Codegen, parseRoutePattern, Alloy } from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Codegen generates TypeScript route code", () => {
  const codegen = new Codegen();
  codegen.addRoute({
    method: "GET",
    path: "/api/hello/<name>",
    name: "hello",
    queryParams: [],
    pathParams: ["name"],
  });

  const output = codegen.generateTs();
  expect(output.routes).toHaveLength(1);
  expect(output.routes[0]?.method).toBe("GET");
  expect(output.routes[0]?.path).toBe("/api/hello/<name>");
  expect(output.tsCode).toContain("/api/hello/<name>");
});

test("parseRoutePattern extracts path and query params", () => {
  const params = parseRoutePattern("/users/<id>?<msg>");
  expect(params["path_params"]).toEqual(["id"]);
  expect(params["query_params"]).toEqual(["msg"]);
});

test("Alloy.file creates a static file marker", () => {
  const f = Alloy.file("./favicon.ico");
  expect(f.path).toBe("./favicon.ico");
});
