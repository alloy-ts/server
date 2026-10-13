import {
  Config,
  file,
  launch,
  registerRoute,
  Response,
  Rocket,
  rocketMain,
  Server,
  serve as nativeServe,
} from "../dist/index.js";

export { Config, file, launch, registerRoute, Response, Rocket, rocketMain, Server };
export * from "./codegen.js";

export function serve(options: any = {}): Server {
  const envPort = process.env["BUN_PORT"] || process.env["PORT"] || process.env["NODE_PORT"];
  const port = options.port !== undefined ? options.port : envPort ? parseInt(envPort, 10) : 3000;
  const hostname = options.hostname || "0.0.0.0";

  return nativeServe({
    ...options,
    port,
    hostname,
  });
}

export const Alloy = {
  serve,
  file,
};

export default Alloy;
