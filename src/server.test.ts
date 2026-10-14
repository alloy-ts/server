import { describe, expect, test } from "vite-plus/test";
import { file, response, responseJson, responseRedirect, Rocket, serve } from "../dist/index.js";

describe("Rocket HTTP Server & Response NAPI Bindings", () => {
  describe("Response constructors & helpers", () => {
    test("response() creates plain response payload", () => {
      const resp = response("Hello, world!", { status: 200 });
      expect(resp.status).toBe(200);
      expect(resp.body).toBeDefined();
      expect(Buffer.from(resp.body!).toString()).toBe("Hello, world!");
    });

    test("responseJson() stringifies data with application/json header", () => {
      const resp = responseJson({ message: "OK" }, { status: 201 });
      expect(resp.status).toBe(201);
      expect(resp.headers["Content-Type"]).toBe("application/json");
      expect(Buffer.from(resp.body!).toString()).toBe(JSON.stringify({ message: "OK" }));
    });

    test("responseRedirect() sets status 302 and Location header", () => {
      const resp = responseRedirect("/target");
      expect(resp.status).toBe(302);
      expect(resp.headers["Location"]).toBe("/target");
    });

    test("file() creates a static file handle", () => {
      const handle = file("./public/index.html");
      expect(handle.path).toBe("./public/index.html");
    });
  });

  describe("serve() function", () => {
    test("serve() initializes server handle with url and port", () => {
      const server = serve(
        {
          "/api/test": response("OK"),
        },
        null,
        { port: 0, hostname: "127.0.0.1" },
      );

      expect(server.url).toContain("http://127.0.0.1:");
      expect(server.port).toBeGreaterThanOrEqual(0);

      // Clean up server
      expect(() => server.stop()).not.toThrow();
    });
  });

  describe("Rocket instance lifecycle", () => {
    test("instantiates Rocket class and allows mounting routes", () => {
      const rocket = new Rocket({
        port: 0,
        address: "127.0.0.1",
      });

      expect(() => {
        rocket.mount("/", "GET", "/status", async () => ({
          status: 200,
          headers: [],
          body: Buffer.from("OK"),
        }));
      }).not.toThrow();

      expect(() => {
        rocket.register("/", 404, async () => ({
          status: 404,
          headers: [],
          body: Buffer.from("Not Found"),
        }));
      }).not.toThrow();
    });
  });
});
