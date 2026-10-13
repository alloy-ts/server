import { expect, test } from "vite-plus/test";
import {
  parseRocketRoutePattern,
  translatePatternToAlloy,
  generateTypescriptClient,
  generateRouteHandler,
  generateTypescriptInterface,
} from "./main.ts";

test("parseRocketRoutePattern parses path and query params", () => {
  const params = parseRocketRoutePattern("/users/<id>/posts/<post_id>?<age>&<name..>");
  expect(params).toHaveLength(4);

  expect(params[0]).toEqual({
    name: "id",
    paramType: "string",
    isPath: true,
    isQuery: false,
    isOptional: false,
  });

  expect(params[1]).toEqual({
    name: "post_id",
    paramType: "string",
    isPath: true,
    isQuery: false,
    isOptional: false,
  });

  expect(params[2]).toEqual({
    name: "age",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: false,
  });

  expect(params[3]).toEqual({
    name: "name",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: true,
  });
});

test("translatePatternToAlloy converts Rocket syntax to Alloy/Bun format", () => {
  expect(translatePatternToAlloy("/users/<id>/<rest..>")).toBe("/users/:id/*");
  expect(translatePatternToAlloy("/files/<path..>")).toBe("/files/*");
  expect(translatePatternToAlloy("/api/<v>/users")).toBe("/api/:v/users");
});

test("generateTypescriptClient generates full client SDK code", () => {
  const result = generateTypescriptClient(
    [
      {
        method: "GET",
        path: "/users/<id>?<age..>",
        name: "getUser",
        returnType: "Promise<{ id: string; name: string }>",
      },
      {
        method: "POST",
        path: "/users",
        name: "createUser",
        returnType: "Promise<{ id: string }>",
      },
    ],
    { clientName: "UserSdk", baseUrl: "https://api.example.com" },
  );

  expect(result.code).toContain("export class UserSdk");
  expect(result.code).toContain('constructor(baseUrl: string = "https://api.example.com")');
  expect(result.code).toContain(
    "async getUser(id: string, age?: string): Promise<{ id: string; name: string }>",
  );
  expect(result.code).toContain("async createUser(): Promise<{ id: string }>");
});

test("generateRouteHandler creates boilerplate function", () => {
  const result = generateRouteHandler({
    method: "POST",
    path: "/posts/<post_id>",
    name: "createPostHandler",
  });

  expect(result.code).toContain(
    "export async function createPostHandler(req: any): Promise<Response>",
  );
  expect(result.code).toContain("const { post_id } = req.params || {};");
  expect(result.code).toContain("return new Response(`Handled POST /posts/<post_id>`);");
});

test("generateTypescriptInterface generates TypeScript interface", () => {
  const result = generateTypescriptInterface("UserProfile", [
    { name: "id", type: "string" },
    { name: "displayName", type: "string" },
    { name: "age", type: "number", optional: true },
    { name: "tags", type: "string[]", optional: true },
  ]);

  expect(result.code).toContain("export interface UserProfile");
  expect(result.code).toContain("id: string;");
  expect(result.code).toContain("displayName: string;");
  expect(result.code).toContain("age?: number;");
  expect(result.code).toContain("tags?: string[];");
});
