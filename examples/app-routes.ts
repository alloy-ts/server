import { App, response } from "../src/main.ts";

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
