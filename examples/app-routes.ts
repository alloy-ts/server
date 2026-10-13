/**
 * Translation of the canonical Rocket quickstart example into TypeScript:
 *
 * ```rust
 * #[macro_use] extern crate rocket;
 *
 * #[get("/")]
 * fn hello() -> &'static str {
 *     "Hello, world!"
 * }
 *
 * #[launch]
 * fn rocket() -> _ {
 *     rocket::build().mount("/", routes![hello])
 * }
 * ```
 *
 * Rocket's macros (`#[get]`, `routes!`, `#[launch]`) are re-expressed at
 * runtime via NAPI bindings (`serve`, `response`).
 */

import { response, serve } from "../dist/index.js";

// #[get("/")]
// fn hello() -> &'static str {
//     "Hello, world!"
// }
//
// Rocket turns the `&'static str` into a `200` `text/plain` response;
// `response(body)` produces a Response with status 200.
const server = serve({
  "/": async () => response("Hello, world!"),
});

console.log(`Server running at ${server.url}`);
