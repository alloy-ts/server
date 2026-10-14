import * as native from "../dist/index.js";

export interface ResponseInit {
  status?: number;
  headers?: Record<string, string>;
}

export class Response {
  status: number;
  headers: Record<string, string>;
  body?: Array<number>;

  constructor(body?: string, init?: ResponseInit) {
    const res = native.response(body, init);
    this.status = res.status;
    this.headers = res.headers;
    this.body = res.body;
  }

  static json(data: any, init?: ResponseInit) {
    return native.responseJson(data, init);
  }

  static redirect(url: string, status?: number) {
    return native.responseRedirect(url, status);
  }
}

export const response = native.response;
export const responseJson = native.responseJson;
export const responseRedirect = native.responseRedirect;
export const outcomeSuccess = native.outcomeSuccess;
export const outcomeError = native.outcomeError;
export const outcomeForward = native.outcomeForward;
export type HandlerOutcome = native.HandlerOutcome;
export const file = native.file;
export const registerRoute = native.registerRoute;
export const Server = native.Server;
export const Route = native.Route;
export type Route = InstanceType<typeof native.Route>;

export interface ServeOptions {
  routes?: Record<string, any>;
  fetch?: (req: any) => any;
  port?: number;
  hostname?: string;
  unix?: string;
}

export function serve(
  optionsOrRoutes: ServeOptions | Record<string, any>,
  options?: native.ServeOptions,
) {
  if (
    optionsOrRoutes &&
    typeof optionsOrRoutes === "object" &&
    ("routes" in optionsOrRoutes || "fetch" in optionsOrRoutes || "port" in optionsOrRoutes)
  ) {
    const opts = optionsOrRoutes as ServeOptions;
    return native.serve(opts.routes, opts.fetch, {
      port: opts.port,
      hostname: opts.hostname,
      unix: opts.unix,
    });
  }
  return native.serve(optionsOrRoutes, null, options);
}

export async function launch(config?: native.ConfigInit) {
  const cfg = config ? native.Config.tryFrom(config) : native.Config.debugDefault();
  return cfg;
}

export class App {
  private routesTable: Record<string, any> = {};
  private serverHandle: native.Server | null = null;

  get(path: string, handler: any): this {
    return this.route("GET", path, handler);
  }

  post(path: string, handler: any): this {
    return this.route("POST", path, handler);
  }

  put(path: string, handler: any): this {
    return this.route("PUT", path, handler);
  }

  delete(path: string, handler: any): this {
    return this.route("DELETE", path, handler);
  }

  patch(path: string, handler: any): this {
    return this.route("PATCH", path, handler);
  }

  head(path: string, handler: any): this {
    return this.route("HEAD", path, handler);
  }

  options(path: string, handler: any): this {
    return this.route("OPTIONS", path, handler);
  }

  route(method: string, path: string, handler: any): this {
    const existing = this.routesTable[path];
    if (!existing) {
      this.routesTable[path] = { [method.toUpperCase()]: handler };
    } else if (
      typeof existing === "object" &&
      !(existing instanceof Response) &&
      typeof existing !== "function"
    ) {
      existing[method.toUpperCase()] = handler;
    } else {
      this.routesTable[path] = { [method.toUpperCase()]: handler };
    }
    return this;
  }

  launch(options?: native.ServeOptions): native.Server {
    this.serverHandle = native.serve(this.routesTable, null, options);
    return this.serverHandle;
  }

  stop(): void {
    if (this.serverHandle) {
      this.serverHandle.stop();
      this.serverHandle = null;
    }
  }
}
