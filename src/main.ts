export * from "../dist/index.js";

import { serve, file } from "../dist/index.js";

export const Alloy = {
  serve,
  file,
};

export function main(): string {
  return "Hello, world!";
}

export default Alloy;
