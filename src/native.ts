import { createRequire } from "node:module";
import { join } from "node:path";
import { existsSync, readdirSync } from "node:fs";

const require = createRequire(import.meta.url);

function getNativeBinding(): any {
  const cwd = process.cwd();
  const searchDirs = [join(cwd, "dist"), cwd];

  for (const dir of searchDirs) {
    if (existsSync(dir)) {
      try {
        const files = readdirSync(dir);
        const nodeFile = files.find((f) => f.endsWith(".node"));
        if (nodeFile) {
          return require(join(dir, nodeFile));
        }
      } catch {
        // Continue searching
      }
    }
  }

  try {
    return require("./index.js");
  } catch {
    throw new Error("Failed to locate native .node binding file.");
  }
}

export const native = getNativeBinding();
