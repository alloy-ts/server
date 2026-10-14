/**
 * Translation of the canonical Rocket quickstart example into TypeScript
 * (`@alloy-ts/server`):
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
 */

import { Response, serve } from "../index.js";

// Rocket's macros (`#[get]`, `routes!`, `#[launch]`) are re-expressed using
// `serve(...)` with route tables and web-standard `Response` objects.

const server = serve({
  port: 3000,
  routes: {
    "/": () => new Response("Hello, world!"),
  },
});

console.log(`Server running at ${server.url}`);
server.stop();
