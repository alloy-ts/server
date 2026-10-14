import { expect, test } from "vite-plus/test";
import { createHandler, HandlerOutcomeStatus, Alloy } from "./main.ts";

test("createHandler creates valid HandlerMetadata with default status", () => {
  const handler = createHandler("custom_handler");
  expect(handler.name).toBe("custom_handler");
  expect(handler.status).toBe(HandlerOutcomeStatus.Success);
});

test("createHandler creates HandlerMetadata with specified outcome status", () => {
  const handler = Alloy.createHandler("error_handler", HandlerOutcomeStatus.Error);
  expect(handler.name).toBe("error_handler");
  expect(handler.status).toBe(HandlerOutcomeStatus.Error);
});
