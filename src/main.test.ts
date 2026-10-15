import { expect, test } from "vite-plus/test";
import {
  response,
  responseJson,
  responseRedirect,
  file,
  Config,
  RocketLifecycle,
  Codegen,
  buildUri,
} from "../dist/index.js";

test("Config API creates and manages configuration", () => {
  const cfg = new Config();
  expect(cfg.port).toBeGreaterThan(0);
  cfg.port = 8080;
  expect(cfg.port).toBe(8080);

  const debug = Config.debugDefault();
  expect(debug.address).toBeDefined();

  const custom = Config.tryFrom({ port: 9090, address: "127.0.0.1" });
  expect(custom.port).toBe(9090);
  expect(custom.address).toBe("127.0.0.1");
});

test("Response constructors work properly", () => {
  const resp1 = response("Hello World", { status: 200 });
  expect(resp1.status).toBe(200);

  const resp2 = responseJson({ message: "Success" });
  expect(resp2.status).toBe(200);
  expect(resp2.headers["Content-Type"]).toBe("application/json");

  const resp3 = responseRedirect("/login");
  expect(resp3.status).toBe(302);
  expect(resp3.headers["Location"]).toBe("/login");

  const staticFile = file("./test.txt");
  expect(staticFile.path).toBe("./test.txt");
});

test("Codegen Rocket attributes parser and uri builder", () => {
  const routeMeta = Codegen.parseRoute("GET", "/person/<name>?<age>");
  expect(routeMeta.method).toBe("GET");
  expect(routeMeta.path).toBe("/person/<name>");
  expect(routeMeta.pathParams[0]?.name).toBe("name");
  expect(routeMeta.queryParams[0]?.name).toBe("age");

  const catchMeta = Codegen.parseCatch("404");
  expect(catchMeta.code).toBe(404);
  expect(catchMeta.isDefault).toBe(false);

  const builtUri = buildUri("/person/<name>?<age>", { name: "John Doe", age: "30" }, "/api");
  expect(builtUri).toBe("/api/person/John%20Doe?age=30");

  const tsRoutes = Codegen.generateTsRoutes([routeMeta]);
  expect(tsRoutes).toContain("GET");

  const formSchema = Codegen.generateFormSchema([
    { name: "username", mappedName: "user_name", validateRule: "len(3..)" },
  ]);
  expect(formSchema).toContain("validateForm");
});

test("Rocket server lifecycle API", () => {
  const rocket = new RocketLifecycle({ port: 8081 });
  expect(rocket).toBeDefined();
});
