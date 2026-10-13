import {
  serve,
  file,
  Response,
  Config,
  Rocket,
  registerRoute,
  launch,
  createRouteMetadata,
  generateRoutesManifest,
  buildUri,
} from "../dist/index.js";

export {
  serve,
  file,
  Response,
  Config,
  Rocket,
  registerRoute,
  launch,
  createRouteMetadata,
  generateRoutesManifest,
  buildUri,
};

export const Alloy = {
  serve,
  file,
};

export function main(): string {
  return "Hello, world!";
}

export default Alloy;
