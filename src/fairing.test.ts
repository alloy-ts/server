import { expect, test } from "vite-plus/test";
import { AdHoc } from "../index.js";

test("fairing.ts - AdHoc constructor and info", () => {
  const fairing = new AdHoc("My Custom Fairing");
  expect(fairing.name).toBe("My Custom Fairing");

  const info = fairing.info();
  expect(info.name).toBe("My Custom Fairing");
  expect(info.ignite).toBe(false);
  expect(info.liftoff).toBe(false);
  expect(info.request).toBe(false);
  expect(info.response).toBe(false);
  expect(info.shutdown).toBe(false);
});

test("fairing.ts - AdHoc static factory constructors", () => {
  const ignite = AdHoc.onIgnite("Ignite Fairing");
  expect(ignite.info().ignite).toBe(true);

  const liftoff = AdHoc.onLiftoff("Liftoff Fairing");
  expect(liftoff.info().liftoff).toBe(true);

  const request = AdHoc.onRequest("Request Fairing");
  expect(request.info().request).toBe(true);

  const response = AdHoc.onResponse("Response Fairing");
  expect(response.info().response).toBe(true);

  const shutdown = AdHoc.onShutdown("Shutdown Fairing");
  expect(shutdown.info().shutdown).toBe(true);
});
