import { expect, test, describe } from "vite-plus/test";
import { Phase, PHASE_BUILD, PHASE_IGNITE, PHASE_ORBIT } from "../dist/index.js";

describe("Phase NAPI bindings", () => {
  test("exposes Phase enum variants and constants", () => {
    expect(Phase.Build()).toBe("Build");
    expect(Phase.Ignite()).toBe("Ignite");
    expect(Phase.Orbit()).toBe("Orbit");
    expect(PHASE_BUILD).toBe("Build");
    expect(PHASE_IGNITE).toBe("Ignite");
    expect(PHASE_ORBIT).toBe("Orbit");
  });
});
