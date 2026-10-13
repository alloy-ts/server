import { Config } from "../src/main.ts";

console.log("=== 1. Inspecting Default Debug Configuration ===");
const debugConfig = Config.debug_default();
console.log(`Profile: ${debugConfig.profile}`);
console.log(`Address: ${debugConfig.address}`);
console.log(`Port: ${debugConfig.port}`);
console.log(`Workers: ${debugConfig.workers}`);
console.log(`Keep Alive: ${debugConfig.keep_alive}s`);

console.log("\n=== 2. Parameters List ===");
console.log("Available Rocket configuration parameters:");
console.log(Config.parameters());

console.log("\n=== 3. Creating Custom Configuration with try_from ===");
const customConfig = Config.try_from({
  profile: "custom",
  address: "127.0.0.1",
  port: 8000,
  workers: 4,
  max_blocking: 256,
  keep_alive: 10,
  cli_colors: true,
});

console.log(`Custom Address: ${customConfig.address}`);
console.log(`Custom Port: ${customConfig.port}`);
console.log(`Custom Workers: ${customConfig.workers}`);

console.log("\n=== 4. Mutating Configuration Getters and Setters ===");
customConfig.port = 9000;
customConfig.keep_alive = 15;
console.log(`Updated Port: ${customConfig.port}`);
console.log(`Updated Keep Alive: ${customConfig.keep_alive}s`);
