import { expect, test } from "vite-plus/test";
import {
  Alloy,
  Config,
  Rocket,
  file,
  generateRouteHandler,
  generateTypescriptClient,
  generateTypescriptInterface,
  main,
  parseRocketRoutePattern,
  response,
  responseJson,
  responseRedirect,
  serve,
  translatePatternToAlloy,
} from "./main.ts";

test("main returns Hello, world!", () => {
  expect(main()).toBe("Hello, world!");
});

test("Config management", () => {
  const config = Config.debugDefault();
  expect(config.port).toBe(8000);
  config.port = 9000;
  expect(config.port).toBe(9000);
  expect(config.address).toBe("127.0.0.1");

  const parameters = Config.parameters();
  expect(parameters).toContain("port");
  expect(parameters).toContain("address");
});

test("Rocket instance creation and route mounting", () => {
  const rocket = new Rocket({ port: 8080 });
  expect(rocket).toBeDefined();
  rocket.mount("/", "GET", "/hello", (_req: unknown) => {
    return response("Hello");
  });
});

test("Response constructors and helpers", () => {
  const plainResp = response("Hello, world!", { status: 200 });
  expect(plainResp.status).toBe(200);

  const jsonResp = responseJson({ ok: true }, { status: 201 });
  expect(jsonResp.status).toBe(201);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redirectResp = responseRedirect("/login", 302);
  expect(redirectResp.status).toBe(302);
  expect(redirectResp.headers["Location"]).toBe("/login");

  const staticFile = file("./test.txt");
  expect(staticFile.path).toBe("./test.txt");
});

test("Alloy serve helper and Server handle", () => {
  const server = Alloy.serve(
    {
      "/status": response("OK"),
    },
    null,
    { port: 9876, hostname: "127.0.0.1" },
  );

  expect(server.url).toBe("http://127.0.0.1:9876");
  expect(server.port).toBe(9876);
  server.stop();

  const directServer = serve(
    {
      "/status": response("OK"),
    },
    null,
    { port: 9877, hostname: "127.0.0.1" },
  );
  expect(directServer.port).toBe(9877);
  directServer.stop();
});

test("parseRocketRoutePattern extracts path and query parameters", () => {
  const params = parseRocketRoutePattern("/users/<id>?<age>&<name..>");
  expect(params).toHaveLength(3);

  const idParam = params.find((p) => p.name === "id");
  expect(idParam).toEqual({
    name: "id",
    paramType: "string",
    isPath: true,
    isQuery: false,
    isOptional: false,
  });

  const ageParam = params.find((p) => p.name === "age");
  expect(ageParam).toEqual({
    name: "age",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: false,
  });

  const nameParam = params.find((p) => p.name === "name");
  expect(nameParam).toEqual({
    name: "name",
    paramType: "string",
    isPath: false,
    isQuery: true,
    isOptional: true,
  });
});

test("translatePatternToAlloy translates Rocket syntax to Alloy/Bun syntax", () => {
  expect(translatePatternToAlloy("/users/<id>/<rest..>")).toBe("/users/:id/*");
  expect(translatePatternToAlloy("/users/<id>?<age>")).toBe("/users/:id");
  expect(translatePatternToAlloy("/posts")).toBe("/posts");
});

test("generateTypescriptClient creates a typed client SDK", () => {
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
  expect(result.code).toContain("/users/${id}");
});

test("generateRouteHandler creates handler boilerplate", () => {
  const result = generateRouteHandler({
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
  });

  expect(result.code).toContain("export async function handleGetUser");
  expect(result.code).toContain("const { id } = req.params;");
});

test("generateTypescriptInterface converts Rust struct or enum to TS", () => {
  const structDef = `
    pub struct User {
      pub id: u64,
      pub name: String,
      pub is_active: bool,
      pub opt: Option<u32>,
    }
  `;

  const interfaceResult = generateTypescriptInterface(structDef);
  expect(interfaceResult.code).toContain("export interface User");
  expect(interfaceResult.code).toContain("id: number;");
  expect(interfaceResult.code).toContain("name: string;");
  expect(interfaceResult.code).toContain("isActive: boolean;");
  expect(interfaceResult.code).toContain("opt?: number;");

  const enumDef = `
    pub enum Role {
      Admin,
      User,
      Guest,
    }
  `;

  const enumResult = generateTypescriptInterface(enumDef);
  expect(enumResult.code).toContain("export type Role =");
  expect(enumResult.code).toContain('"Admin"');
  expect(enumResult.code).toContain('"User"');
  expect(enumResult.code).toContain('"Guest"');
});
