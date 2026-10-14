/**
 * Example: Custom Handler and Request Handler Outcomes
 *
 * Demonstrates:
 * 1. Creating `Handler` wrappers for custom Rocket request handlers
 * 2. Producing `Outcome.success`, `Outcome.error`, and `Outcome.forward`
 * 3. Inspecting outcome status codes and response bodies
 */

import { Handler, Outcome, OutcomeStatus } from "../dist/index.js";

// 1. Instantiating custom Handler plugin wrapper
const myHandler = new Handler("my-custom-plugin");
console.log("Handler Type:", myHandler.handlerType);

// 2. Creating a successful request handling Outcome (200 OK)
const successOutcome = Outcome.success(Buffer.from("Hello from custom handler!"));
console.log(
  `Success Outcome - Code: ${successOutcome.code}, Status: ${successOutcome.status} (Success)`,
);
console.log("Response Body:", Buffer.from(successOutcome.body!).toString("utf-8"));

// 3. Creating an Error Outcome (403 Forbidden)
const errorOutcome = Outcome.error(403);
console.log(`Error Outcome - Code: ${errorOutcome.code}, Status: ${errorOutcome.status} (Error)`);

// 4. Creating a Forward Outcome (Pass request to next matching route)
const forwardOutcome = Outcome.forward(404);
console.log(
  `Forward Outcome - Code: ${forwardOutcome.code}, Status: ${forwardOutcome.status} (Forward)`,
);

console.log("Handler Outcomes example executed successfully.");
