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

// ---------------------------------------------------------------------------
// Header Normalization & Response
// ---------------------------------------------------------------------------

export interface HeaderInput {
  name: string;
  value: string;
}

export function normalizeHeaders(
  headers?: [string, string][] | Record<string, string> | HeaderInput[] | null,
): HeaderInput[] {
  if (!headers) return [];
  if (Array.isArray(headers)) {
    return headers.map((h) => {
      if (Array.isArray(h)) {
        return { name: String(h[0]), value: String(h[1]) };
      }
      return { name: String(h.name), value: String(h.value) };
    });
  }
  return Object.entries(headers).map(([name, value]) => ({
    name: String(name),
    value: String(value),
  }));
}

/**
 * JS-friendly Response object mirroring Rocket / Bun Response.
 * Deserializes directly into Rust's ResponsePayload.
 */
export class Response {
  status: number;
  headers: HeaderInput[];
  body?: Uint8Array | number[];

  constructor(
    body?: any,
    init?: {
      status?: number;
      headers?: [string, string][] | Record<string, string> | HeaderInput[] | null;
    },
  ) {
    this.status = init?.status ?? 200;
    this.headers = normalizeHeaders(init?.headers);

    if (body !== undefined && body !== null) {
      if (typeof body === "string") {
        this.body = Array.from(Buffer.from(body));
      } else if (Buffer.isBuffer(body) || body instanceof Uint8Array) {
        this.body = Array.from(body);
      } else if (Array.isArray(body)) {
        this.body = body;
      } else {
        this.body = Array.from(Buffer.from(JSON.stringify(body)));
      }
    }
  }

  static json(
    data: any,
    init?: {
      status?: number;
      headers?: [string, string][] | Record<string, string> | HeaderInput[] | null;
    },
  ) {
    const headers = normalizeHeaders(init?.headers);
    if (!headers.some((h) => h.name.toLowerCase() === "content-type")) {
      headers.push({ name: "Content-Type", value: "application/json" });
    }
    return new Response(JSON.stringify(data), {
      status: init?.status ?? 200,
      headers,
    });
  }

  static redirect(url: string, status = 302) {
    return new Response(null, {
      status,
      headers: [{ name: "Location", value: url }],
    });
  }
}

export const response = (
  body?: any,
  init?: {
    status?: number;
    headers?: [string, string][] | Record<string, string> | HeaderInput[] | null;
  },
) => new Response(body, init);

export const responseJson = (data: any, init?: any) => Response.json(data, init);
export const responseRedirect = (url: string, status?: number) => Response.redirect(url, status);

// ---------------------------------------------------------------------------
// Native exports & Server ergonomics
// ---------------------------------------------------------------------------

export const serve = (routesOrOptions?: any, fetchFn?: any, optionsArg?: any) => {
  let routes = routesOrOptions;
  let fetch = fetchFn;
  let options = optionsArg;

  // Support single-object signature: Alloy.serve({ routes, fetch, port, hostname, unix })
  if (
    routesOrOptions &&
    typeof routesOrOptions === "object" &&
    !Array.isArray(routesOrOptions) &&
    fetchFn === undefined &&
    optionsArg === undefined &&
    (routesOrOptions.routes !== undefined ||
      routesOrOptions.fetch !== undefined ||
      routesOrOptions.port !== undefined ||
      routesOrOptions.hostname !== undefined ||
      routesOrOptions.unix !== undefined)
  ) {
    routes = routesOrOptions.routes;
    fetch = routesOrOptions.fetch;
    options = {
      port: routesOrOptions.port,
      hostname: routesOrOptions.hostname,
      unix: routesOrOptions.unix,
    };
  }

  if (native.serve) {
    return native.serve(routes, fetch, options);
  }
  return {
    url: `http://${options?.hostname || "0.0.0.0"}:${options?.port || 3000}`,
    port: options?.port || 3000,
    stop() {},
  };
};

export const file = (path: string) => {
  if (native.file) {
    return native.file(path);
  }
  return { path };
};

export const launch = native.launch;
export const registerRoute = (method: string, path: string, handler: any) => {
  recordRouteMeta(method, path);
  if (native.registerRoute) {
    return native.registerRoute(method, path, handler);
  }
};

export const Config = native.Config;
export const Server = native.Server;

// ---------------------------------------------------------------------------
// App Builder Abstraction
// ---------------------------------------------------------------------------

export class App {
  private routes: Record<string, any> = {};

  private addRoute(method: string, path: string, handler: any) {
    const existing = this.routes[path];
    if (!existing) {
      this.routes[path] = { [method]: handler };
    } else if (typeof existing === "object" && !Array.isArray(existing)) {
      existing[method] = handler;
    } else {
      this.routes[path] = { GET: existing, [method]: handler };
    }
  }

  get(path: string, handler: any) {
    this.addRoute("GET", path, handler);
    return this;
  }

  post(path: string, handler: any) {
    this.addRoute("POST", path, handler);
    return this;
  }

  put(path: string, handler: any) {
    this.addRoute("PUT", path, handler);
    return this;
  }

  delete(path: string, handler: any) {
    this.addRoute("DELETE", path, handler);
    return this;
  }

  patch(path: string, handler: any) {
    this.addRoute("PATCH", path, handler);
    return this;
  }

  head(path: string, handler: any) {
    this.addRoute("HEAD", path, handler);
    return this;
  }

  options(path: string, handler: any) {
    this.addRoute("OPTIONS", path, handler);
    return this;
  }

  launch(options?: any) {
    return serve(this.routes, undefined, options);
  }
}

// ---------------------------------------------------------------------------
// Route Registry & Decorators
// ---------------------------------------------------------------------------

export interface RouteMeta {
  method: string;
  path: string;
}

const registeredRoutesMeta: RouteMeta[] = [];

function recordRouteMeta(method: string, path: string) {
  registeredRoutesMeta.push({ method: method.toUpperCase(), path });
}

export function getRegisteredRoutesMeta(): RouteMeta[] {
  return [...registeredRoutesMeta];
}

/**
 * Decorator and helper for declaring route handlers.
 */
export function Route(method: string, path: string) {
  recordRouteMeta(method, path);
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor,
  ): PropertyDescriptor {
    const originalMethod = descriptor.value;
    if (typeof originalMethod === "function" && native.registerRoute) {
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

export function Catch(_code?: number) {
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor,
  ): PropertyDescriptor {
    return descriptor;
  };
}

export const route = (method: string, path: string, handler: any) =>
  registerRoute(method, path, handler);
export const get = (path: string, handler: any) => registerRoute("GET", path, handler);
export const post = (path: string, handler: any) => registerRoute("POST", path, handler);
export const put = (path: string, handler: any) => registerRoute("PUT", path, handler);
export const del = (path: string, handler: any) => registerRoute("DELETE", path, handler);
export const deleteRoute = (path: string, handler: any) => registerRoute("DELETE", path, handler);
export const patch = (path: string, handler: any) => registerRoute("PATCH", path, handler);
export const head = (path: string, handler: any) => registerRoute("HEAD", path, handler);
export const options = (path: string, handler: any) => registerRoute("OPTIONS", path, handler);

// ---------------------------------------------------------------------------
// Rocket Codegen & Type-safe URI construction
// ---------------------------------------------------------------------------

export interface RouteParam {
  name: string;
  isTrailing: boolean;
}

export interface RouteInfo {
  method: string;
  uri: string;
  pathParams: RouteParam[];
  queryParams: string[];
}

/**
 * Parses a Rocket route URI pattern like `/users/<id>/<action..>?<ref>` or `/users/:id`.
 */
export function parseRoutePattern(pattern: string): RouteInfo {
  const pathParams: RouteParam[] = [];
  const queryParams: string[] = [];

  const [pathPart, queryPart] = pattern.split("?");

  if (pathPart) {
    for (const seg of pathPart.split("/").filter(Boolean)) {
      if (seg.startsWith(":")) {
        pathParams.push({ name: seg.slice(1), isTrailing: false });
      } else if (seg.startsWith("<") && seg.endsWith(">")) {
        const inner = seg.slice(1, -1);
        if (inner.endsWith("..")) {
          pathParams.push({ name: inner.slice(0, -2), isTrailing: true });
        } else {
          pathParams.push({ name: inner, isTrailing: false });
        }
      }
    }
  }

  if (queryPart) {
    for (const seg of queryPart.split("&").filter(Boolean)) {
      if (seg.startsWith(":")) {
        queryParams.push(seg.slice(1));
      } else if (seg.startsWith("<") && seg.endsWith(">")) {
        const inner = seg.slice(1, -1);
        const name = inner.endsWith("..") ? inner.slice(0, -2) : inner;
        queryParams.push(name);
      } else if (seg.includes("=")) {
        const [key] = seg.split("=");
        if (key) queryParams.push(key);
      } else {
        queryParams.push(seg);
      }
    }
  }

  return {
    method: "GET",
    uri: pattern,
    pathParams,
    queryParams,
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

/**
 * Helper to build route URI from pattern and parameter dictionary.
 */
export function buildRouteUri(pattern: string, params?: Record<string, unknown>): string {
  if (!params) return pattern;
  const [pathPart, queryPart] = pattern.split("?");
  let path = uri(pathPart || "", params);

  if (queryPart) {
    const queryPairs: string[] = [];
    for (const seg of queryPart.split("&").filter(Boolean)) {
      let paramName = seg;
      if (seg.startsWith(":")) paramName = seg.slice(1);
      else if (seg.startsWith("<") && seg.endsWith(">")) {
        const inner = seg.slice(1, -1);
        paramName = inner.endsWith("..") ? inner.slice(0, -2) : inner;
      } else if (seg.includes("=")) {
        paramName = seg.split("=")[0] || seg;
      }

      if (params[paramName] !== undefined) {
        queryPairs.push(
          `${encodeURIComponent(paramName)}=${encodeURIComponent(String(params[paramName]))}`,
        );
      }
    }
    if (queryPairs.length > 0) {
      path += `?${queryPairs.join("&")}`;
    }
  }

  return path;
}

/**
 * Codegen helper to generate TypeScript interface definitions for registered routes.
 */
export function generateRoutesTypeDefinitions(patterns: string[]): string {
  let code = "// Generated Rocket Route Types\n\n";
  code += "export interface ApiRoutes {\n";

  for (const pattern of patterns) {
    const info = parseRoutePattern(pattern);
    code += `  "${info.method} ${info.uri}": {\n`;
    code += "    params: {\n";
    for (const p of info.pathParams) {
      code += `      ${p.name}: string;\n`;
    }
    code += "    };\n";
    code += "    query: {\n";
    for (const q of info.queryParams) {
      code += `      ${q}?: string;\n`;
    }
    code += "    };\n";
    code += "  };\n";
  }

  code += "}\n";
  return code;
}

/**
 * Alloy namespace exposing Rocket server runtime and helper utilities.
 */
export const Alloy = {
  serve,
  file,
  launch,
  registerRoute,
  Response,
  Config,
  Server,
  App,
  Route,
  Get,
  Post,
  Put,
  Delete,
  Patch,
  Head,
  Options,
  Catch,
  uri,
  buildRouteUri,
  parseRoutePattern,
  generateRoutesTypeDefinitions,
  getRegisteredRoutesMeta,
};

export default Alloy;
