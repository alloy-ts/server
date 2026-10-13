import { expect, test } from "vite-plus/test";
import {
  main,
  parseRocketRoutePattern,
  translatePatternToAlloy,
  generateTypescriptClient,
  generateRouteHandler,
  generateTypescriptInterface,
  Config,
  response,
  responseJson,
  responseRedirect,
  file,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("parseRocketRoutePattern extracts dynamic path and query parameters", () => {
  const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
  expect(params).toEqual([
    {
      name: "id",
      paramType: "string",
      isPath: true,
      isQuery: false,
      isOptional: false,
    },
    {
      name: "age",
      paramType: "string",
      isPath: false,
      isQuery: true,
      isOptional: false,
    },
    {
      name: "name",
      paramType: "string",
      isPath: false,
      isQuery: true,
      isOptional: true,
    },
  ]);
});

test("translatePatternToAlloy converts Rocket route syntax to Alloy/Bun route syntax", () => {
  expect(translatePatternToAlloy("/users/<id>/<rest..>")).toBe("/users/:id/*");
  expect(translatePatternToAlloy("/api/posts/<post_id>")).toBe("/api/posts/:post_id");
});

test("generateTypescriptClient produces a fetch client SDK", () => {
  const result = generateTypescriptClient(
    [
      {
        method: "GET",
        path: "/users/<id>",
        name: "getUser",
        params: [
          {
            name: "id",
            paramType: "string",
            isPath: true,
            isQuery: false,
            isOptional: false,
          },
        ],
        returnType: "Promise<{ id: string; name: string }>",
      },
    ],
    { clientName: "UserApiClient" },
  );

  expect(result.code).toContain("export class UserApiClient");
  expect(result.code).toContain("async getUser(id: string)");
  expect(result.code).toContain("fetch");
});

test("generateRouteHandler produces handler boilerplate", () => {
  const result = generateRouteHandler({
    method: "GET",
    path: "/users/<id>",
    name: "getUserHandler",
  });

  expect(result.code).toContain("export async function getUserHandler(req: any)");
  expect(result.code).toContain("const { id } = req.params || {};");
});

test("generateTypescriptInterface converts field specs to interface", () => {
  const result = generateTypescriptInterface("User", [
    { name: "id", type: "string" },
    { name: "age", type: "number", optional: true },
  ]);

  expect(result.code).toContain("export interface User");
  expect(result.code).toContain("id: string;");
  expect(result.code).toContain("age?: number;");
});

test("Config class operates properly", () => {
  const config = Config.debugDefault();
  expect(config.port).toBe(8000);
  config.port = 3000;
  expect(config.port).toBe(3000);

  const initConfig = Config.tryFrom({ port: 8080, address: "127.0.0.1" });
  expect(initConfig.port).toBe(8080);
  expect(initConfig.address).toBe("127.0.0.1");
});

test("response helpers create Response object", () => {
  const res1 = response("OK", { status: 200 });
  expect(res1.status).toBe(200);

  const res2 = responseJson({ ok: true });
  expect(res2.status).toBe(200);
  expect(res2.headers["Content-Type"]).toBe("application/json");

  const res3 = responseRedirect("/login");
  expect(res3.status).toBe(302);
  expect(res3.headers["Location"]).toBe("/login");

  const staticFile = file("./test.txt");
  expect(staticFile.path).toBe("./test.txt");
});
