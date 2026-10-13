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

import { registerRoute, serve, response, type Server } from "../dist/index.js";

export class App {
  private port: number = 3000;
  private hostname: string = "0.0.0.0";
  private serverHandle?: Server;

  constructor(options?: { port?: number; hostname?: string }) {
    if (options?.port) this.port = options.port;
    if (options?.hostname) this.hostname = options.hostname;
  }

  get(path: string, handler: (req: any) => Promise<any> | any): this {
    registerRoute("GET", path, async (req: any) => {
      const res = await handler(req);
      return typeof res === "string" ? response(res) : res;
    });
    return this;
  }

  post(path: string, handler: (req: any) => Promise<any> | any): this {
    registerRoute("POST", path, async (req: any) => {
      const res = await handler(req);
      return typeof res === "string" ? response(res) : res;
    });
    return this;
  }

  launch(): Server {
    this.serverHandle = serve(undefined, undefined, {
      port: this.port,
      hostname: this.hostname,
    });
    return this.serverHandle;
  }

  stop(): void {
    this.serverHandle?.stop();
  }
}

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

if (process.env.NODE_ENV === "test") {
  app.stop();
}
