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

import { serve, response, responseJson, responseRedirect, file } from "../dist/index.js";

const server = serve({
  routes: {
    "/": () => response("Hello, world!"),
    "/api/status": responseJson({ status: "OK" }),
    "/users/:id": (req: any) => response(`Hello User ${req.params?.id}!`),
    "/blog/hello": responseRedirect("/blog/hello/world"),
    "/favicon.ico": file("./favicon.ico"),
  },
});

console.log(`Server running at ${server.url}`);
