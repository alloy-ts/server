import { expect, test } from "vite-plus/test";
import { AdHoc } from "./main.ts";

test("AdHoc.onIgnite creates ignite fairing instance", () => {
  const fairing = AdHoc.onIgnite("IgniteFairing", () => {});
  expect(fairing.name).toBe("IgniteFairing");
  expect(fairing.info.kind).toBe(1);
});

test("AdHoc.onLiftoff creates liftoff fairing instance", () => {
  const fairing = AdHoc.onLiftoff("LiftoffFairing", () => {});
  expect(fairing.name).toBe("LiftoffFairing");
  expect(fairing.info.kind).toBe(2);
});
