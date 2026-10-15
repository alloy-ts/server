import { expect, test } from "vite-plus/test";
import { Config, Rocket, Route } from "../dist/index.js";

test("Rocket.build creates build phase instance", () => {
  const rocket = Rocket.build();
  expect(rocket.phase).toBe(0);
});

test("Rocket.custom configures rocket instance", () => {
  const config = Config.debug_default();
  const rocket = Rocket.custom(config);
  expect(rocket.phase).toBe(0);
  expect(rocket.config()).toBeDefined();
});

test("Rocket phase transitions", () => {
  const rocket = Rocket.build();
  expect(rocket.phase).toBe(0);

  const route = new Route("GET", "/test", "test_handler", 1, "json");
  rocket.mount("/api", [route]);

  rocket.ignite();
  expect(rocket.phase).toBe(1);

  rocket.launch();
  expect(rocket.phase).toBe(2);

  rocket.shutdown();
  expect(rocket.phase).toBe(0);
});
