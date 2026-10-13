import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const nativeBinding = require("./dist/alloy-server.node");

export const {
  Config,
  Response,
  Server,
  ADDRESS,
  CLI_COLORS,
  DEBUG_PROFILE,
  DEFAULT_PROFILE,
  IDENT,
  IP_HEADER,
  KEEP_ALIVE,
  LIMITS,
  LOG_LEVEL,
  MAX_BLOCKING,
  PORT,
  RELEASE_PROFILE,
  SECRET_KEY,
  SHUTDOWN,
  TEMP_DIR,
  TLS,
  WORKERS,
  file,
  generateTsRouteManifest,
  generateUri,
  launch,
  parseCatchAttribute,
  parseRouteAttribute,
  registerRoute,
  serve,
} = nativeBinding;

export const __napiBindingTarget = "native";
