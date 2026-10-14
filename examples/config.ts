import { Config } from "../dist/index.js";

console.log("=== 1. Inspecting Default Debug Configuration ===");
const debugConfig = Config.debug_default();
console.log(`Profile: ${debugConfig.profile}`);
console.log(`Address: ${debugConfig.address}`);
console.log(`Port: ${debugConfig.port}`);
console.log(`Workers: ${debugConfig.workers}`);
console.log(`Keep Alive: ${debugConfig.keepAlive}s`);

console.log("\n=== 2. Parameters List ===");
console.log("Available Rocket configuration parameters:");
console.log(Config.parameters());

console.log("\n=== 3. Creating Custom Configuration with try_from ===");
const customConfig = Config.try_from({
  profile: "custom",
  address: "127.0.0.1",
  port: 8000,
  workers: 4,
  maxBlocking: 256,
  keepAlive: 10,
  cliColors: true,
});

console.log(`Custom Address: ${customConfig.address}`);
console.log(`Custom Port: ${customConfig.port}`);
console.log(`Custom Workers: ${customConfig.workers}`);

console.log("\n=== 4. Mutating Configuration Getters and Setters ===");
customConfig.port = 9000;
customConfig.keepAlive = 15;
console.log(`Updated Port: ${customConfig.port}`);
console.log(`Updated Keep Alive: ${customConfig.keepAlive}s`);
