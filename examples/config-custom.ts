import { Config, Alloy, Response } from "../src/main.ts";

// Create custom Rocket configuration
const customConfig = Config.try_from({
  port: 8080,
  address: "127.0.0.1",
  workers: 4,
  logLevel: "normal",
  limits: {
    json: "5MiB",
    form: "128KiB",
  },
});

console.log("Configured Rocket parameters:");
console.log(`- Address: ${customConfig.address}`);
console.log(`- Port: ${customConfig.port}`);
console.log(`- Workers: ${customConfig.workers}`);
console.log(`- Log Level: ${customConfig.logLevel}`);

const server = Alloy.serve(
  {
    "/": new Response("Server running with custom configuration!"),
  },
  null,
  { port: customConfig.port, hostname: customConfig.address },
);

console.log(`Server listening on ${server.url}`);
