import { expect, test } from "vite-plus/test";
import { Response, serve } from "../dist/index.js";

test("serve creates a Rocket server with routes and option config", async () => {
  const routes = {
    "/api/status": {
      GET: async () => Response.json({ status: "ok" }),
    },
  };

  const server = serve(routes, undefined, { port: 0, hostname: "127.0.0.1" });
  expect(server.port).toBeGreaterThan(0);
  expect(server.url).toContain("http://127.0.0.1:");

  const res = await fetch(`${server.url}/api/status`);
  expect(res.status).toBe(200);
  const data = await res.json();
  expect(data).toEqual({ status: "ok" });

  server.stop();
});
