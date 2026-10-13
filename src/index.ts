export interface AppOptions {
  port?: number;
  hostname?: string;
  unix?: string;
  fetch?: (req: unknown) => unknown;
}

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

/**
 * High-level builder and application launcher mirroring Rocket's `rocket::build()`.
 */
export class App {
  private routesTable: Record<string, unknown> = {};
  private activeServer: ReturnType<typeof serve> | null = null;

  private addRoute(method: string, path: string, handler: unknown): this {
    const current = this.routesTable[path];
    if (current && typeof current === "object" && !("status" in current) && !("path" in current)) {
      (current as Record<string, unknown>)[method.toUpperCase()] = handler;
    } else {
      this.routesTable[path] = { [method.toUpperCase()]: handler };
    }
    return this;
  }

  get(path: string, handler: unknown): this {
    return this.addRoute("GET", path, handler);
  }

  post(path: string, handler: unknown): this {
    return this.addRoute("POST", path, handler);
  }

  put(path: string, handler: unknown): this {
    return this.addRoute("PUT", path, handler);
  }

  delete(path: string, handler: unknown): this {
    return this.addRoute("DELETE", path, handler);
  }

  patch(path: string, handler: unknown): this {
    return this.addRoute("PATCH", path, handler);
  }

  head(path: string, handler: unknown): this {
    return this.addRoute("HEAD", path, handler);
  }

  options(path: string, handler: unknown): this {
    return this.addRoute("OPTIONS", path, handler);
  }

  mount(basePath: string, routes: Record<string, unknown>): this {
    const cleanBase = basePath.endsWith("/") ? basePath.slice(0, -1) : basePath;
    for (const [pattern, handler] of Object.entries(routes)) {
      const fullPath = `${cleanBase}${pattern.startsWith("/") ? pattern : `/${pattern}`}`;
      this.routesTable[fullPath] = handler;
    }
    return this;
  }

  launch(options?: AppOptions) {
    const server = serve({
      routes: this.routesTable,
      ...options,
    });
    this.activeServer = server;
    return server;
  }

  stop(): void {
    if (this.activeServer) {
      this.activeServer.stop();
      this.activeServer = null;
    }
  }
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
  App,
};

/** Bun.serve API compatibility namespace */
export const Bun = {
  serve,
  file,
};

export * from "../dist/index.js";
