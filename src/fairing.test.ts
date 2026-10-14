import { expect, test } from "vite-plus/test";
import { Fairing } from "../dist/index.js";

test("Fairing constructor and info getter", () => {
  const fairing = new Fairing("CustomFairing");
  expect(fairing.name).toBe("CustomFairing");

  const info = fairing.info();
  expect(info.name).toBe("CustomFairing");
  expect(info.kindIgnite).toBe(true);
  expect(info.kindRequest).toBe(true);
});
