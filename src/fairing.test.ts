import { expect, test } from "vite-plus/test";
import {
  Fairing,
  KIND_IGNITE,
  KIND_LIFTOFF,
  KIND_REQUEST,
  KIND_RESPONSE,
  KIND_SHUTDOWN,
  KIND_SINGLETON,
} from "../index.js";

test("Fairing constants", () => {
  expect(KIND_IGNITE).toBe(1);
  expect(KIND_LIFTOFF).toBe(2);
  expect(KIND_REQUEST).toBe(4);
  expect(KIND_RESPONSE).toBe(8);
  expect(KIND_SHUTDOWN).toBe(16);
  expect(KIND_SINGLETON).toBe(32);
});

test("Fairing constructor and info getter", () => {
  const mask = KIND_REQUEST | KIND_RESPONSE;
  const f = new Fairing("Request/Response Counter", mask);
  expect(f.name).toBe("Request/Response Counter");
  expect(f.kind).toBe(mask);

  const info = f.info();
  expect(info.name).toBe("Request/Response Counter");
  expect(info.kind).toBe(mask);
});

test("Fairing ad_hoc constructor", () => {
  const f = Fairing.adHoc("Custom AdHoc Fairing", KIND_IGNITE);
  expect(f.name).toBe("Custom AdHoc Fairing");
  expect(f.kind).toBe(KIND_IGNITE);
});
