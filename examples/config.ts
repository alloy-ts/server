import { Alloy, Config, response } from "../src/main.ts";

// 1. Create a Config with debug defaults
const config = new Config();
console.log("Default Debug Config:");
console.log(`- Profile: ${config.profile}`);
console.log(`- Address: ${config.address}`);
console.log(`- Port: ${config.port}`);
console.log(`- Workers: ${config.workers}`);
console.log(`- Max Blocking Threads: ${config.maxBlocking}`);
console.log(`- Log Level: ${config.logLevel}`);

// 2. Custom configuration overrides using Config.tryFrom()
const customConfig = Config.tryFrom({
  port: 8080,
  address: "127.0.0.1",
  workers: 4,
  maxBlocking: 256,
  ident: "AlloyRocketServer",
  logLevel: "normal",
  limits: {
    limits: {
      json: "10MiB",
      form: "64KiB",
    },
  },
  shutdown: {
    ctrlc: true,
    grace: 2,
    mercy: 3,
    force: true,
    signals: ["term"],
  },
});

console.log("\nCustom Config:");
console.log(`- Address: ${customConfig.address}:${customConfig.port}`);
console.log(`- Ident: ${customConfig.ident}`);
console.log(`- Temp Dir: ${customConfig.tempDir}`);

// 3. Launch server with custom port and configuration options
const server = Alloy.serve({
  port: customConfig.port,
  hostname: customConfig.address,
  routes: {
    "/": response("Server running with custom Rocket Config!"),
  },
});

console.log(`\nServer launched at ${server.url}`);

setTimeout(() => {
  server.stop();
  console.log("Server stopped successfully.");
}, 1000);
