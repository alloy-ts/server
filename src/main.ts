import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

let native: any;
try {
  native = require("../dist/alloy-server.linux-x64-gnu.node");
} catch {
  try {
    native = require("../dist/index.js");
  } catch {
    native = {};
  }
}

/**
 * Main entrypoint function for basic setup & testing.
 */
export function main(): string {
  return "Hello, world!";
}

export const serve = native.serve;
export const file = native.file;
export const response = native.response;
export const responseJson = native.responseJson;
export const responseRedirect = native.responseRedirect;
export const registerRoute = native.registerRoute;
export const Config = native.Config;
export const Rocket = native.Rocket;
export const launch = native.launch;
export const rocketMain = native.rocketMain;
export const parseRouteUri = native.parseRouteUri;
export const generateTsUriBuilder = native.generateTsUriBuilder;
export const parseCatcherStatus = native.parseCatcherStatus;
export const translatePattern = native.translatePattern;

/**
 * High-level Alloy framework interface for TypeScript / Node.js
 * wrapping Rocket's NAPI engine.
 */
export const Alloy = {
  /**
   * Launch a high-performance Rocket HTTP server with Bun/Alloy-style routes or fetch handler.
   */
  serve(options: {
    routes?: Record<string, any>;
    fetch?: (req: any) => any;
    port?: number;
    hostname?: string;
    unix?: string;
  }): any {
    const { routes, fetch, ...opts } = options;
    return native.serve ? native.serve(routes, fetch, opts) : null;
  },

  /**
   * Lazily serve a file from disk.
   */
  file(path: string): any {
    return native.file ? native.file(path) : { path };
  },

  /**
   * Helper to generate typed TS code for `uri!` helper generation.
   */
  generateUriBuilder(routes: any[]): string {
    return native.generateTsUriBuilder ? native.generateTsUriBuilder(routes) : "";
  },
};

// ---------------------------------------------------------------------------
// TypeScript Decorators for Rocket Code Generation
// ---------------------------------------------------------------------------

const registeredDecoratedRoutes: Array<{ method: string; path: string; handler: any }> = [];

function createRouteDecorator(method: string) {
  return function (path: string) {
    return function (originalMethod: any, context: ClassMethodDecoratorContext) {
      const handler = originalMethod;
      void context;
      registeredDecoratedRoutes.push({ method, path, handler });
      if (native.registerRoute) {
        native.registerRoute(method, path, handler);
      }
      return originalMethod;
    };
  };
}

export const get = createRouteDecorator("GET");
export const post = createRouteDecorator("POST");
export const put = createRouteDecorator("PUT");
export const del = createRouteDecorator("DELETE");
export const patch = createRouteDecorator("PATCH");

export function catchError(status: number | string) {
  return function (originalMethod: any, _context: ClassMethodDecoratorContext) {
    const parsedStatus =
      typeof status === "number"
        ? status
        : native.parseCatcherStatus
          ? native.parseCatcherStatus(status)
          : null;
    void parsedStatus;
    return originalMethod;
  };
}
