import { expect, test } from "vite-plus/test";
import { Rocket, BuildPhase, IgnitePhase, OrbitPhase } from "../dist/index.js";

test("Phase structs instantiation and properties", () => {
  const buildPhase = new BuildPhase();
  expect(buildPhase.routeCount).toBe(0);
  expect(buildPhase.catcherCount).toBe(0);

  const ignitePhase = new IgnitePhase();
  expect(ignitePhase.isIgnited).toBe(true);

  const orbitPhase = new OrbitPhase();
  expect(orbitPhase.isOrbiting).toBe(true);
});

test("Rocket static constructors build and custom", () => {
  const r1 = Rocket.build();
  expect(r1).toBeInstanceOf(Rocket);

  const r2 = Rocket.custom({ port: 9000 });
  expect(r2).toBeInstanceOf(Rocket);
});

test("Rocket prototype methods manage, state, configure, attach, ignite, config, routes, catchers", () => {
  const r = new Rocket();

  r.manage("key", "value");
  expect(r.state("key")).toBe("value");

  r.attach("MyFairing");
  r.configure({ port: 9090 });
  r.ignite();

  expect(r.figment()).toBe("figment");

  const config = r.config();
  expect(config.port).toBe(8000);

  expect(Array.isArray(r.routes())).toBe(true);
  expect(Array.isArray(r.catchers())).toBe(true);
});
