import { execSync } from "child_process";
import { copyFileSync, existsSync, mkdirSync, readdirSync } from "fs";

const args = process.argv.slice(2);
const isRelease = args.includes("--release") || args.includes("-r");

const buildCmd = isRelease ? "cargo build --release" : "cargo build";
execSync(buildCmd, { stdio: "inherit" });

const targetDir = isRelease ? "./target/release" : "./target/debug";

if (!existsSync("./dist")) {
  mkdirSync("./dist", { recursive: true });
}

if (existsSync(targetDir)) {
  const files = readdirSync(targetDir);
  const nativeLib = files.find(
    (f) =>
      (f.startsWith("liballoy_server_native") || f.startsWith("alloy_server_native")) &&
      (f.endsWith(".so") || f.endsWith(".dylib") || f.endsWith(".dll") || f.endsWith(".node")),
  );
  if (nativeLib) {
    const src = `${targetDir}/${nativeLib}`;
    copyFileSync(src, "./alloy-server.linux-x64-gnu.node");
    copyFileSync(src, "./dist/alloy-server.linux-x64-gnu.node");
  }
}
