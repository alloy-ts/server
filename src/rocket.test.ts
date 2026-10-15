import { expect, test } from "vite-plus/test";
import { Catcher, Config, Rocket, Route } from "../dist/index.js";

test("Rocket can be instantiated via constructor or static factory methods", () => {
  const r1 = new Rocket();
  expect(r1.phase).toBe("Build");

  const customConfig = Config.debugDefault();
  customConfig.port = 8080;

  const r2 = new Rocket(customConfig);
  expect(r2.config().port).toBe(8080);

  const r3 = new Rocket();
  expect(r3.phase).toBe("Build");

  const r4 = new Rocket(customConfig);
  expect(r4.config().port).toBe(8080);
});

test("Rocket mounts routes, registers catchers, manages state, and launches", () => {
  const rocket = new Rocket();

  const route = new Route("GET", "/hello");
  rocket.mount("/api", [route]);
  expect(rocket.routes()).toHaveLength(1);
  expect(rocket.routes()[0]!.uri).toBe("/api/hello");

  const catcher = new Catcher(404, "/");
  rocket.register("/v1", [catcher]);
  expect(rocket.catchers()).toHaveLength(1);

  rocket.manage("db_url", "postgres://localhost/test");
  expect(rocket.state("db_url")).toBe("postgres://localhost/test");

  rocket.ignite();
  expect(rocket.phase).toBe("Ignite");

  const handle = Rocket.execute(rocket);
  expect(handle.url).toContain("http://127.0.0.1:");
  expect(rocket.phase).toBe("Orbit");
});
