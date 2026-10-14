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

import { Rocket, Response } from "../dist/index.js";

// #[get("/")]
// fn hello() -> &'static str {
//     "Hello, world!"
// }
//
// Rocket turns the `&'static str` into a `200` `text/plain` response;
// `new Response(body)` does the same on the Alloy side.
const rocket = new Rocket();

rocket.mount("/", "GET", "/", async () => new Response("Hello, world!"));

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", routes![hello])
// }
//
// `launch` builds, ignites, and starts listening on a
// dedicated background runtime; it returns immediately, like `#[launch]`.
rocket.launch();

console.log("Server running at http://localhost:3000");
