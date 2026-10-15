import { expect, test } from "vite-plus/test";
import { Rocket, Route, Catcher } from "../dist/index.js";

test("Rocket.build, custom, and constructor initialization", () => {
  const defaultRocket = Rocket.build();
  expect(defaultRocket.phase).toBe(0 /* Build */);
  expect(defaultRocket.config.port).toBeGreaterThan(0);

  const customRkt = Rocket.custom({ port: 8088, address: "127.0.0.1" });
  expect(customRkt.config.port).toBe(8088);
  expect(customRkt.config.address).toBe("127.0.0.1");

  const ctorRocket = new Rocket({ port: 9090 });
  expect(ctorRocket.config.port).toBe(9090);

  expect(Rocket.execute()).toContain("started");
});

test("Rocket mount routes, register catchers, and phase transitions", () => {
  const rocket = Rocket.build();

  const r1 = new Route("GET", "/world", "hi");
  rocket.mount("/hello", [r1]);

  const mountedRoutes = rocket.routes();
  expect(mountedRoutes.length).toBe(1);
  expect(mountedRoutes[0]?.uri).toBe("/hello/world");

  const c1 = new Catcher(404, "not_found");
  rocket.register("/api", [c1]);

  const registeredCatchers = rocket.catchers();
  expect(registeredCatchers.length).toBe(1);
  expect(registeredCatchers[0]?.base).toBe("/api/");

  expect(rocket.phase).toBe(0 /* Build */);
  rocket.ignite();
  expect(rocket.phase).toBe(1 /* Ignite */);
  rocket.launch();
  expect(rocket.phase).toBe(2 /* Orbit */);

  rocket.manage("DatabasePool");
  expect(rocket.state("DatabasePool")).toBe(true);
  expect(rocket.state("RedisPool")).toBe(false);
});
