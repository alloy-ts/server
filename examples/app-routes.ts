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
 */

import { Alloy, response } from "../dist/index.js";

// #[get("/")]
// fn hello() -> &'static str {
//     "Hello, world!"
// }

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", routes![hello])
// }
const server = Alloy.serve({
  routes: {
    "/": () => response("Hello, world!"),
  },
});

console.log(`Server running at ${server.url}`);
