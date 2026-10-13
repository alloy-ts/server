export * from "../dist/index.js";

import {
  file,
  generateClient,
  generateTypes,
  parseRoute,
  response,
  responseJson,
  responseRedirect,
  serve as nativeServe,
  toRocketPath,
} from "../dist/index.js";
import type { ServeOptions } from "../dist/index.js";

export const parseRocketRoutePattern = parseRoute;
export const translatePatternToAlloy = toRocketPath;
export const generateTypescriptClient = generateClient;
export const generateTypescriptInterface = generateTypes;

function wrapRoutes(routes: unknown): unknown {
  if (!routes || typeof routes !== "object") return routes;
  const wrapped: Record<string, unknown> = {};
  for (const [pattern, handler] of Object.entries(routes as Record<string, unknown>)) {
    if (typeof handler === "function") {
      const fn = handler as (...args: unknown[]) => unknown;
      wrapped[pattern] = async (...args: unknown[]) => await fn(...args);
    } else if (
      handler &&
      typeof handler === "object" &&
      !("status" in handler) &&
      !("path" in handler)
    ) {
      const methodMap: Record<string, unknown> = {};
      for (const [method, fn] of Object.entries(handler as Record<string, unknown>)) {
        if (typeof fn === "function") {
          const f = fn as (...args: unknown[]) => unknown;
          methodMap[method] = async (...args: unknown[]) => await f(...args);
        } else {
          methodMap[method] = fn;
        }
      }
      wrapped[pattern] = methodMap;
    } else {
      wrapped[pattern] = handler;
    }
  }
  return wrapped;
}

function wrapFetch(fetchFn: unknown): unknown {
  if (typeof fetchFn !== "function") return fetchFn;
  const fn = fetchFn as (...args: unknown[]) => unknown;
  return async (...args: unknown[]) => await fn(...args);
}

interface ServeConfig {
  routes?: unknown;
  fetch?: unknown;
  port?: number;
  hostname?: string;
  unix?: string;
  [key: string]: unknown;
}

export function serve(routes?: unknown, fetch?: unknown, options?: unknown) {
  if (
    routes &&
    typeof routes === "object" &&
    ("routes" in routes || "fetch" in routes || "port" in routes || "unix" in routes)
  ) {
    const opts = { ...(routes as ServeConfig) };
    if (opts.routes) opts.routes = wrapRoutes(opts.routes);
    if (opts.fetch) opts.fetch = wrapFetch(opts.fetch);
    return nativeServe(opts);
  }
  return nativeServe(wrapRoutes(routes), wrapFetch(fetch), options as ServeOptions | undefined);
}

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
