import { expect, test } from "vite-plus/test";
import { PhaseState } from "../dist/index.js";

test("PhaseState constructs phase markers for Build, Ignite, and Orbit", () => {
  const build = PhaseState.build();
  expect(build.current).toBe("Build");
  expect(build.toString()).toBe("Build");

  const ignite = PhaseState.ignite();
  expect(ignite.current).toBe("Ignite");

  const orbit = PhaseState.orbit();
  expect(orbit.current).toBe("Orbit");
});
