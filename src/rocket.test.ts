import { expect, test, describe } from "vite-plus/test";
import { Rocket, Config, Route, Catcher, Handler, Phase } from "../dist/index.js";

describe("Rocket NAPI bindings", () => {
  test("supports Rocket.build() and default phase", () => {
    const rocket = Rocket.build();
    expect(rocket.phase).toBe(Phase.Build());
  });

  test("supports new Rocket() constructor", () => {
    const rocket = new Rocket();
    expect(rocket.phase).toBe(Phase.Build());
    expect(rocket.config().port).toBe(8000);
  });

  test("supports new Rocket(config) and Rocket.custom(config)", () => {
    const config = Config.debugDefault();
    config.port = 9000;
    const rocket1 = new Rocket(config);
    expect(rocket1.config().port).toBe(9000);

    const rocket2 = Rocket.custom(config);
    expect(rocket2.config().port).toBe(9000);
  });

  test("supports mounting routes and registering catchers", () => {
    const rocket = Rocket.build();
    const route = new Route("GET", "/hello");
    const catcher = new Catcher(404, new Handler(() => "not found"));

    rocket.mount("/api", [route]);
    rocket.register("/api", [catcher]);

    expect(rocket.routes().length).toBe(1);
    expect(rocket.catchers().length).toBe(1);
  });

  test("supports managing state and transitioning phases", () => {
    const rocket = Rocket.build();
    rocket.manage("db", "postgres://localhost/mydb");
    expect(rocket.state("db")).toBe("postgres://localhost/mydb");

    const ignited = rocket.ignite();
    expect(ignited.phase).toBe(Phase.Ignite());

    const launched = ignited.launch();
    expect(launched.phase).toBe(Phase.Orbit());

    const executed = Rocket.execute(launched);
    expect(executed.phase).toBe(Phase.Orbit());
  });
});
