import { describe, expect, test } from "vite-plus/test";
import { serve } from "../dist/index.js";

describe("serve function", () => {
  test("creates Server instance via serve", () => {
    const server = serve();
    expect(typeof server.url).toBe("string");
    expect(typeof server.port).toBe("number");
    server.stop();
  });
});
