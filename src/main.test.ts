import { expect, test } from "vite-plus/test";
import {
  Config,
  Rocket,
  file,
  response,
  responseJson,
  responseRedirect,
  serve,
  uri,
} from "./main.ts";

test("Response constructors and helpers", () => {
  const resp = response("Hello World", { status: 200 });
  expect(resp.status).toBe(200);
  expect(resp.body).toBeDefined();

  const jsonResp = responseJson({ key: "value" });
  expect(jsonResp.status).toBe(200);
  expect(jsonResp.headers["Content-Type"]).toBe("application/json");

  const redir = responseRedirect("https://example.com", 301);
  expect(redir.status).toBe(301);
  expect(redir.headers["Location"]).toBe("https://example.com");

  const f = file("./test.txt");
  expect(f.path).toBe("./test.txt");
});

test("Config API", () => {
  const cfg = new Config();
  expect(cfg.port).toBe(8000);
  cfg.port = 9000;
  expect(cfg.port).toBe(9000);

  const debugDefaults = Config.debugDefault();
  expect(debugDefaults.profile).toBe("debug");
});

test("URI codegen helper", () => {
  const result = uri("/users/:id", { id: "123" }, { active: "true" });
  expect(result).toBe("/users/123?active=true");
});

test("Server instantiation via Rocket and serve", () => {
  const rocket = new Rocket({ port: 8080 });
  expect(rocket).toBeDefined();

  const server = serve(
    {
      "/health": () => response("OK"),
    },
    undefined,
    { port: 8080 },
  );
  expect(server.port).toBe(8080);
  server.stop();
});
