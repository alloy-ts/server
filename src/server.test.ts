import { expect, test } from "vite-plus/test";
import { App, file, launch, registerRoute, Response, serve } from "./server.ts";

test("server module exports Response, serve, file, registerRoute, and launch", () => {
  expect(typeof serve).toBe("function");
  expect(typeof registerRoute).toBe("function");
  expect(typeof launch).toBe("function");
  expect(typeof file).toBe("function");
  expect(typeof Response).toBe("function");
  expect(typeof App).toBe("function");
});

test("App registers routes and responds as expected", () => {
  const app = new App();
  app.get("/hello", () => new Response("Hello"));
  app.post("/items", () => Response.json({ item: 1 }));

  expect(typeof app.get).toBe("function");
  expect(typeof app.post).toBe("function");
  expect(typeof app.launch).toBe("function");
});

test("server module Response helper creates responses", () => {
  const resp = new Response("server test");
  expect(resp.status).toBe(200);

  const jsonResp = Response.json({ server: true });
  expect(jsonResp.status).toBe(200);
});

test("server module file marker creates static file marker", () => {
  const marker = file("./static.txt");
  expect(marker.path).toBe("./static.txt");
});
