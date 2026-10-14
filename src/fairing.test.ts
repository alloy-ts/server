import { expect, test } from "vite-plus/test";
import {
  createAdhocFairing,
  createFairingKind,
  requestFairingKind,
  responseFairingKind,
} from "../dist/index.js";

test("FairingKind helper functions create bitset flags", () => {
  const requestKind = requestFairingKind();
  expect(requestKind.request).toBe(true);
  expect(requestKind.response).toBe(false);

  const customKind = createFairingKind(true, false, true, true, false, false);
  expect(customKind.ignite).toBe(true);
  expect(customKind.request).toBe(true);
  expect(customKind.response).toBe(true);
});

test("createAdhocFairing creates named fairing metadata handles", () => {
  const kind = responseFairingKind();
  const fairing = createAdhocFairing("Request Logger", kind);

  expect(fairing.name).toBe("Request Logger");
  expect(fairing.kind.response).toBe(true);
});
