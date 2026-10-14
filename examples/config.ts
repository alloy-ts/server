import { Config, Alloy, response } from "../dist/index.js";

// Demonstrate Rocket NAPI configuration bindings
const config = new Config();
config.port = 8080;
config.address = "127.0.0.1";
config.logLevel = "normal";

console.log(`Configured profile: ${config.profile}`);
console.log(`Address: ${config.address}`);
console.log(`Port: ${config.port}`);

const server = Alloy.serve({
  port: config.port,
  hostname: config.address,
  routes: {
    "/": response("Configured Rocket Server"),
  },
});

console.log(`Server running at ${server.url}`);
