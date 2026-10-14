import * as native from "../dist/index.js";

export function main(): string {
  return "Hello, world!";
}

export interface AppOptions {
  port?: number;
  hostname?: string;
  unix?: string;
}

export class App {
  private routesMap: Record<string, Record<string, unknown>> = {};
  private configOptions: AppOptions;
  private serverInstance: native.Server | null = null;

  constructor(options: AppOptions = {}) {
    this.configOptions = options;
  }

  public route(method: string, path: string, handler: unknown): this {
    const m = method.toUpperCase();
    if (!this.routesMap[path]) {
      this.routesMap[path] = {};
    }
    this.routesMap[path][m] = handler;
    return this;
  }

  public get(path: string, handler: unknown): this {
    return this.route("GET", path, handler);
  }

  public post(path: string, handler: unknown): this {
    return this.route("POST", path, handler);
  }

  public put(path: string, handler: unknown): this {
    return this.route("PUT", path, handler);
  }

  public delete(path: string, handler: unknown): this {
    return this.route("DELETE", path, handler);
  }

  public patch(path: string, handler: unknown): this {
    return this.route("PATCH", path, handler);
  }

  public head(path: string, handler: unknown): this {
    return this.route("HEAD", path, handler);
  }

  public options(path: string, handler: unknown): this {
    return this.route("OPTIONS", path, handler);
  }

  public mount(base: string, routes: Record<string, unknown>): this {
    for (const [path, handler] of Object.entries(routes)) {
      const fullPath =
        base === "/"
          ? path
          : `${base.replace(/\/$/, "")}${path.startsWith("/") ? path : `/${path}`}`;
      this.routesMap[fullPath] = {
        ...this.routesMap[fullPath],
        ...(typeof handler === "object" && handler !== null ? handler : { GET: handler }),
      };
    }
    return this;
  }

  public launch(): native.Server {
    this.serverInstance = native.serve(this.routesMap, null, this.configOptions);
    return this.serverInstance;
  }

  public stop(): void {
    if (this.serverInstance) {
      this.serverInstance.stop();
      this.serverInstance = null;
    }
  }
}

export const Alloy = {
  serve(options: {
    routes?: unknown;
    fetch?: unknown;
    port?: number;
    hostname?: string;
    unix?: string;
  }) {
    const { routes, fetch, ...opts } = options;
    return native.serve(routes, fetch, opts);
  },
  file: native.file,
};

export const serve = native.serve;
export const file = native.file;
export const response = native.response;
export const responseJson = native.responseJson;
export const responseRedirect = native.responseRedirect;
export const registerRoute = native.registerRoute;
export const Codegen = native.Codegen;
export const parseRoutePattern = native.parseRoutePattern;
export const Config = native.Config;
export const Route = native.Route;
export const CustomHandler = native.CustomHandler;

export { native };
