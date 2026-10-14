import { Rocket, response } from "../dist/index.js";

const rocket = new Rocket({ port: 3000, address: "127.0.0.1" });

rocket.mount("/", "GET", "/", async () => response("Hello, world!"));

void rocket.launch();

console.log("Server running at http://localhost:3000");
