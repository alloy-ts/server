import { describe, expect, it } from "vite-plus/test";
import {
  file,
  registerRoute,
  response,
  responseJson,
  responseRedirect,
  serve,
  Server,
} from "../dist/index.js";

describe("server exports", () => {
  it("response helpers construct response objects", () => {
    const res = response("Hello", { status: 200, headers: { "X-Test": "1" } });
    expect(res.status).toBe(200);
    expect(res.headers).toEqual({ "X-Test": "1" });
    expect(res.body).toEqual(Array.from(Buffer.from("Hello")));

    const jsonRes = responseJson({ message: "ok" });
    expect(jsonRes.status).toBe(200);
    expect(jsonRes.headers).toEqual({ "Content-Type": "application/json" });

    const redirectRes = responseRedirect("https://example.com", 301);
    expect(redirectRes.status).toBe(301);
    expect(redirectRes.headers).toEqual({ Location: "https://example.com" });

    const f = file("./favicon.ico");
    expect(f.path).toBe("./favicon.ico");
  });

  it("registerRoute function accepts route definition", () => {
    expect(() => {
      registerRoute("GET", "/test", async () => response("OK"));
    }).not.toThrow();
  });

  it("serve starts a server and returns a Server handle", () => {
    const server = serve(
      {
        "/ping": () => response("pong"),
      },
      undefined,
      { port: 0 },
    );

    expect(server).toBeInstanceOf(Server);
    expect(server.url).toBe("http://0.0.0.0:0");
    expect(typeof server.port).toBe("number");

    server.stop();
  });
});
