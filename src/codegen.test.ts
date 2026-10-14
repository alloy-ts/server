import { expect, test } from "vite-plus/test";
import { Codegen, parseRoutePattern } from "../dist/index.js";

test("Codegen accumulates routes and generates TS code output", () => {
  const codegen = new Codegen();

  codegen.addRoute({
    method: "get",
    path: "/person/<name>?<age>",
    name: "person",
    rank: 1,
    format: "json",
    queryParams: ["age"],
    pathParams: ["name"],
  });

  codegen.addRoute({
    method: "post",
    path: "/person",
    name: "create_person",
    dataParam: "person_data",
    queryParams: [],
    pathParams: [],
  });

  const result = codegen.generateTs();

  expect(result.routes).toHaveLength(2);
  expect(result.routes[0]?.method).toBe("get");
  expect(result.routes[0]?.name).toBe("person");
  expect(result.routes[1]?.method).toBe("post");
  expect(result.routes[1]?.dataParam).toBe("person_data");

  expect(result.tsCode).toContain('"/person/<name>?<age>"');
  expect(result.tsCode).toContain("GET: async (req: any)");
  expect(result.tsCode).toContain('"/person"');
  expect(result.tsCode).toContain("POST: async (req: any)");
});

test("parseRoutePattern parses path and query parameter names accurately", () => {
  const parsed1 = parseRoutePattern("/person/<name>/<rest..>?<age>&<msg..>");
  expect(parsed1["path_params"]).toEqual(["name", "rest"]);
  expect(parsed1["query_params"]).toEqual(["age", "msg"]);

  const parsed2 = parseRoutePattern("/users/:id/posts/:postId?sort=:sort");
  expect(parsed2["path_params"]).toEqual(["id", "postId"]);
  expect(parsed2["query_params"]).toEqual(["sort"]);
});
