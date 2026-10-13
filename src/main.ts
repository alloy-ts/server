import {
  Config,
  file,
  launch,
  registerRoute,
  Response,
  Rocket,
  rocketMain,
  Server,
  serve,
} from "../dist/index.js";

export { Config, file, launch, registerRoute, Response, Rocket, rocketMain, Server, serve };
export * from "./codegen.js";

export const Alloy = {
  serve,
  file,
};

export default Alloy;
