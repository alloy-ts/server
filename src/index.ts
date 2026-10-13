export * from "../dist/index.js";

import { generateClient, generateTypes, parseRoute, toRocketPath } from "../dist/index.js";

export const parseRocketRoutePattern = parseRoute;
export const translatePatternToAlloy = toRocketPath;
export const generateTypescriptClient = generateClient;
export const generateTypescriptInterface = generateTypes;
