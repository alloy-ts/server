import { file, launch, registerRoute, Response, serve, Server } from "../index.js";
import type { RequestInput, ResponseInit, ResponsePayload } from "../index.js";

export { file, launch, registerRoute, Response, serve, Server };

export const DEBUG_PROFILE = "debug";
export const RELEASE_PROFILE = "release";
export const DEFAULT_PROFILE = "debug";

export interface ConfigInit {
  profile?: string;
  address?: string;
  port?: number;
  workers?: number;
  maxBlocking?: number;
  ident?: string;
  ipHeader?: string;
  limits?: { limits?: Record<string, string> };
  tempDir?: string;
  keepAlive?: number;
  shutdown?: {
    ctrlc?: boolean;
    signals?: string[];
    grace?: number;
    mercy?: number;
    force?: boolean;
  };
  logLevel?: string;
  cliColors?: boolean;
}

/**
 * NAPI / JS mirror of Rocket's `Config`.
 */
export class Config {
  profile = "debug";
  address = "127.0.0.1";
  port = 8000;
  workers = 4;
  maxBlocking = 512;
  ident: string | null = "Rocket";
  ipHeader: string | null = "X-Real-IP";
  limits = { limits: {} };
  tempDir = "/tmp";
  keepAlive = 5;
  shutdown = { ctrlc: true, signals: ["term", "hup"], grace: 5, mercy: 5, force: false };
  logLevel = "normal";
  cliColors = true;

  static debugDefault(): Config {
    return new Config();
  }

  static releaseDefault(): Config {
    const cfg = new Config();
    cfg.profile = "release";
    cfg.logLevel = "critical";
    return cfg;
  }

  static defaultConfig(): Config {
    return new Config();
  }

  static parameters(): string[] {
    return [
      "address",
      "port",
      "workers",
      "max_blocking",
      "keep_alive",
      "ident",
      "ip_header",
      "limits",
      "tls",
      "secret_key",
      "temp_dir",
      "log_level",
      "shutdown",
      "cli_colors",
    ];
  }

  static tryFrom(init: ConfigInit): Config {
    const cfg = new Config();
    if (init.profile !== undefined) cfg.profile = init.profile;
    if (init.address !== undefined) cfg.address = init.address;
    if (init.port !== undefined) cfg.port = init.port;
    if (init.workers !== undefined) cfg.workers = init.workers;
    if (init.maxBlocking !== undefined) cfg.maxBlocking = init.maxBlocking;
    if (init.ident !== undefined) cfg.ident = init.ident;
    if (init.ipHeader !== undefined) cfg.ipHeader = init.ipHeader;
    if (init.tempDir !== undefined) cfg.tempDir = init.tempDir;
    if (init.keepAlive !== undefined) cfg.keepAlive = init.keepAlive;
    if (init.logLevel !== undefined) cfg.logLevel = init.logLevel;
    if (init.cliColors !== undefined) cfg.cliColors = init.cliColors;
    return cfg;
  }
}

export function main() {
  return "Hello, world!";
}

/**
 * Helper to construct a `Response` instance.
 */
export function response(body?: unknown, init?: ResponseInit): Response {
  return new Response(body, init);
}

/**
 * App class that re-expresses Rocket's build, route, and launch operations at runtime.
 */
export class App {
  get(
    path: string,
    handler: (req: RequestInput) => ResponsePayload | Promise<ResponsePayload>,
  ): this {
    registerRoute("GET", path, async (req) => handler(req));
    return this;
  }

  post(
    path: string,
    handler: (req: RequestInput) => ResponsePayload | Promise<ResponsePayload>,
  ): this {
    registerRoute("POST", path, async (req) => handler(req));
    return this;
  }

  put(
    path: string,
    handler: (req: RequestInput) => ResponsePayload | Promise<ResponsePayload>,
  ): this {
    registerRoute("PUT", path, async (req) => handler(req));
    return this;
  }

  delete(
    path: string,
    handler: (req: RequestInput) => ResponsePayload | Promise<ResponsePayload>,
  ): this {
    registerRoute("DELETE", path, async (req) => handler(req));
    return this;
  }

  launch(): Promise<void> {
    return launch();
  }
}

/**
 * Codegen / Decorator support for Rocket routes in TypeScript.
 */
export function get(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("GET", path, descriptor.value);
  };
}

export function post(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("POST", path, descriptor.value);
  };
}

export function put(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("PUT", path, descriptor.value);
  };
}

export function del(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("DELETE", path, descriptor.value);
  };
}

/**
 * Type-safe URI builder helper, mirroring Rocket's `uri!` macro for codegen.
 */
export function uri(path: string, params: Record<string, string | number> = {}): string {
  let result = path;
  for (const [key, val] of Object.entries(params)) {
    result = result.replace(`:${key}`, encodeURIComponent(String(val)));
    result = result.replace(`<${key}>`, encodeURIComponent(String(val)));
  }
  return result;
}

/**
 * High-performance Alloy server facade powered by Rocket NAPI bindings.
 */
export const Alloy = {
  serve,
  file,
  Response,
};
