import { execSync } from "child_process";
import { copyFileSync, existsSync, mkdirSync } from "fs";

const args = process.argv.slice(2);
const isRelease = args.includes("--release") || args.includes("-r");

const buildCmd = isRelease ? "cargo build --release" : "cargo build";
execSync(buildCmd, { stdio: "inherit" });

const debugSo = "./target/debug/liballoy_server_native.so";
const releaseSo = "./target/release/liballoy_server_native.so";

if (!existsSync("./dist")) {
  mkdirSync("./dist", { recursive: true });
}

if (existsSync(releaseSo)) {
  copyFileSync(releaseSo, "./alloy-server.linux-x64-gnu.node");
  copyFileSync(releaseSo, "./dist/alloy-server.linux-x64-gnu.node");
} else if (existsSync(debugSo)) {
  copyFileSync(debugSo, "./alloy-server.linux-x64-gnu.node");
  copyFileSync(debugSo, "./dist/alloy-server.linux-x64-gnu.node");
}
