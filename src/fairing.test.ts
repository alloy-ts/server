import { expect, test } from "vite-plus/test";
import {
  type AdHocFairing,
  type FairingInfo,
  type FairingKind,
  adHocFairing,
  createFairingInfo,
} from "../dist/index.js";

test("FairingKind enum values", () => {
  expect(1 as FairingKind).toBe(1); // Ignite
  expect(2 as FairingKind).toBe(2); // Liftoff
  expect(4 as FairingKind).toBe(4); // Request
  expect(8 as FairingKind).toBe(8); // Response
  expect(16 as FairingKind).toBe(16); // Shutdown
  expect(32 as FairingKind).toBe(32); // Singleton
});

test("createFairingInfo constructor", () => {
  const info: FairingInfo = createFairingInfo("Request Logger", 4 | 8);
  expect(info.name).toBe("Request Logger");
  expect(info.kind).toBe(12); // Request (4) | Response (8)
});

test("adHocFairing creation and default kind", () => {
  const fairing1: AdHocFairing = adHocFairing("CORS Middleware");
  expect(fairing1.name).toBe("CORS Middleware");
  expect(fairing1.kind).toBe(12); // Default: Request | Response

  const fairing2: AdHocFairing = adHocFairing("Ignition Fairing", 1);
  expect(fairing2.name).toBe("Ignition Fairing");
  expect(fairing2.kind).toBe(1); // Ignite
});
