/**
 * Translation of the canonical Rocket quickstart example into Alloy
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
 *
 * Rocket's macros (`#[get]`, `routes!`, `#[launch]`) cannot cross the N-API
 * boundary, so the `App` class re-expresses them at runtime. Its `get`/`post`/
 * `…`/`launch`/`stop` methods are the 1:1 counterparts of `rocket::build()`,
 * `#[get("/")]`, and `#[launch]`:
 *
 *   `rocket::build()`        -> `new App()`
 *   `#[get("/")] fn hello`   -> `app.get("/", …)`
 *   `routes![hello]`         -> (the handler passed to `app.get`)
 *   `#[launch] … .mount(..)` -> `app.launch()`
 */

import { App, response } from "@alloy-ts/server";

// #[get("/")]
// fn hello() -> &'static str {
//     "Hello, world!"
// }
//
// Rocket turns the `&'static str` into a `200` `text/plain` response;
// `response(body)` does the same on the Alloy side (status defaults to 200).
const app = new App();

app.get("/", async () => response("Hello, world!"));

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", routes![hello])
// }
//
// `launch` builds, ignites, and starts listening (default 0.0.0.0:3000) on a
// dedicated background runtime; it returns immediately, like `#[launch]`.
app.launch();

console.log("Server running at http://localhost:3000");

// Stop server gracefully after demo
setTimeout(() => {
  app.stop();
  console.log("App server stopped gracefully");
}, 1000);
