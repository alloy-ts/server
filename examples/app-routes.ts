import { registerRoute, Response, serve } from "../dist/index.js";

// #[get("/")]
// fn hello() -> &'static str {
//     "Hello, world!"
// }
//
// Rocket turns the `&'static str` into a `200` `text/plain` response;
// `new Response(body)` does the same on the Alloy side (status defaults to 200).
registerRoute("GET", "/", async () => new Response("Hello, world!"));

// #[launch]
// fn rocket() -> _ {
//     rocket::build().mount("/", routes![hello])
// }
//
// `serve` builds, ignites, and starts listening (default 0.0.0.0:3000) on a
// dedicated background runtime; it returns immediately, like `#[launch]`.
const server = serve(null, null, { port: 3000 });

console.log(`Server running at ${server.url}`);
