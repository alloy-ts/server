export * from "../dist/index.js";

import {
  file,
  generateClient,
  generateTypes,
  parseRoute,
  response,
  responseJson,
  responseRedirect,
  serve,
  toRocketPath,
} from "../dist/index.js";

export const parseRocketRoutePattern = parseRoute;
export const translatePatternToAlloy = toRocketPath;
export const generateTypescriptClient = generateClient;
export const generateTypescriptInterface = generateTypes;

/** Bun / Alloy compatible Response constructor & helpers */
export const ResponseHelper = {
  json: responseJson,
  redirect: responseRedirect,
};

/** High-performance Alloy server API backed by Rocket web framework */
export const Alloy = {
  serve,
  file,
  response,
  Response: ResponseHelper,
};

/** Bun.serve API compatibility namespace */
export const Bun = {
  serve,
  file,
};
