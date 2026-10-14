import { createRequire } from "node:module";

const require = createRequire(import.meta.url);

function loadNative(): any {
  try {
    return require("./index.js");
  } catch {
    try {
      return require("../dist/index.js");
    } catch {
      return {};
    }
  }
}

const native = loadNative();

/**
 * Main entrypoint function returning standard greeting.
 */
export function main(): string {
  return "Hello, world!";
}

/**
 * Convenience helper to create a Response object.
 */
export function response(body?: string, init?: any): any {
  return new native.Response(body, init);
}

/**
 * App class mirroring Rocket's builder pattern (rocket::build()).
 */
export class App {
  private routes: Array<{ method: string; path: string; handler: any }> = [];

  get(path: string, handler: any): this {
    this.routes.push({ method: "GET", path, handler });
    native.registerRoute("GET", path, handler);
    return this;
  }

  post(path: string, handler: any): this {
    this.routes.push({ method: "POST", path, handler });
    native.registerRoute("POST", path, handler);
    return this;
  }

  put(path: string, handler: any): this {
    this.routes.push({ method: "PUT", path, handler });
    native.registerRoute("PUT", path, handler);
    return this;
  }

  delete(path: string, handler: any): this {
    this.routes.push({ method: "DELETE", path, handler });
    native.registerRoute("DELETE", path, handler);
    return this;
  }

  patch(path: string, handler: any): this {
    this.routes.push({ method: "PATCH", path, handler });
    native.registerRoute("PATCH", path, handler);
    return this;
  }

  head(path: string, handler: any): this {
    this.routes.push({ method: "HEAD", path, handler });
    native.registerRoute("HEAD", path, handler);
    return this;
  }

  options(path: string, handler: any): this {
    this.routes.push({ method: "OPTIONS", path, handler });
    native.registerRoute("OPTIONS", path, handler);
    return this;
  }

  mount(base: string, method: string, path: string, handler: any): this {
    const fullPath =
      base.endsWith("/") && path.startsWith("/") ? base + path.slice(1) : base + path;
    this.routes.push({ method, path: fullPath, handler });
    native.registerRoute(method, fullPath, handler);
    return this;
  }

  launch(): Promise<void> {
    return native.launch();
  }
}

/**
 * Re-exported NAPI bindings for direct usage.
 */
export const serve = native.serve;
export const file = native.file;
export const launch = native.launch;
export const registerRoute = native.registerRoute;
export const Response = native.Response;
export const Config = native.Config;
export const Server = native.Server;
export const RocketRoute = native.Route;
export const RocketHandler = native.Handler;

/**
 * Alloy namespace exposing Rocket server runtime and helper utilities.
 */
export const Alloy = {
  serve: native.serve,
  file: native.file,
  launch: native.launch,
  registerRoute: native.registerRoute,
  Response: native.Response,
  Config: native.Config,
  Server: native.Server,
  Route: native.Route,
  Handler: native.Handler,
  App,
  response,
};

// ---------------------------------------------------------------------------
// Rocket Codegen support for TypeScript
// ---------------------------------------------------------------------------

/**
 * Generic route decorator for declaring route handlers on class methods.
 */
export function Route(method: string, path: string) {
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor,
  ): PropertyDescriptor {
    const originalMethod = descriptor.value;
    if (typeof originalMethod === "function") {
      native.registerRoute(method, path, originalMethod);
    }
    return descriptor;
  };
}

export function Get(path: string) {
  return Route("GET", path);
}

export function Post(path: string) {
  return Route("POST", path);
}

export function Put(path: string) {
  return Route("PUT", path);
}

export function Delete(path: string) {
  return Route("DELETE", path);
}

export function Patch(path: string) {
  return Route("PATCH", path);
}

export function Head(path: string) {
  return Route("HEAD", path);
}

export function Options(path: string) {
  return Route("OPTIONS", path);
}

/**
 * Catcher decorator for error handling.
 */
export function Catch(_code?: number) {
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor,
  ): PropertyDescriptor {
    return descriptor;
  };
}

/**
 * Type-safe and URL-safe route URI constructor.
 * Replaces parameters like `:id`, `<id>`, or `<id..>` with encoded values.
 */
export function uri(pattern: string, params?: Record<string, unknown>): string {
  if (!params) return pattern;
  let result = pattern;
  for (const [key, value] of Object.entries(params)) {
    const valStr =
      typeof value === "object" && value !== null
        ? JSON.stringify(value)
        : value !== undefined && value !== null
          ? String(value as string | number | boolean)
          : "";
    const encoded = encodeURIComponent(valStr);
    result = result
      .replace(new RegExp(`:${key}\\b`, "g"), encoded)
      .replace(new RegExp(`<${key}\\.\\.>`, "g"), encoded)
      .replace(new RegExp(`<${key}>`, "g"), encoded);
  }
  return result;
}

export default Alloy;
