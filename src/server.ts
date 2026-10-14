import { native } from "./native.ts";

export interface RocketConfig {
  address?: string;
  port?: number;
  workers?: number;
  maxBlocking?: number;
  keepAlive?: number;
  ident?: string;
  logLevel?: string;
  shutdown?: {
    grace?: number;
    mercy?: number;
    force?: boolean;
  };
}

export interface ResponseInit {
  status?: number;
  headers?: Record<string, string>;
}

export interface ServeOptions {
  port?: number;
  hostname?: string;
  unix?: string;
  routes?: Record<string, any>;
  fetch?: (req: any) => any;
}

export function file(path: string): any {
  return native.file(path);
}

export function registerRoute(method: string, path: string, handler: unknown): void {
  return native.registerRoute(method, path, handler);
}

export function launch(config?: RocketConfig): Promise<void> {
  return native.launch(config);
}

export const Rocket = native.Rocket;
export const Server = native.Server;

export function response(body?: string | null, init?: ResponseInit | null): any {
  return native.response(body, init);
}

export function responseJson(data: any, init?: ResponseInit | null): any {
  return native.responseJson(data, init);
}

export function responseRedirect(url: string, status?: number | null): any {
  return native.responseRedirect(url, status);
}

export class Response {
  status: number;
  headers: Record<string, string>;
  body?: number[];

  constructor(body?: string | null, init?: ResponseInit | null) {
    const res = native.response(body, init);
    this.status = res.status;
    this.headers = res.headers;
    this.body = res.body;
  }

  static json(data: any, init?: ResponseInit | null): any {
    return native.responseJson(data, init);
  }

  static redirect(url: string, status?: number | null): any {
    return native.responseRedirect(url, status);
  }
}

export function serve(routesOrOptions?: any, fetchFn?: any, opts?: ServeOptions): any {
  if (
    routesOrOptions &&
    typeof routesOrOptions === "object" &&
    !("GET" in routesOrOptions || "POST" in routesOrOptions) &&
    (routesOrOptions.routes !== undefined ||
      routesOrOptions.port !== undefined ||
      routesOrOptions.fetch !== undefined ||
      routesOrOptions.hostname !== undefined ||
      routesOrOptions.unix !== undefined)
  ) {
    const { routes, fetch, ...scalarOpts } = routesOrOptions;
    return native.serve(routes, fetch, scalarOpts);
  }
  return native.serve(routesOrOptions, fetchFn, opts);
}

export class App {
  private routes: Record<string, Record<string, any>> = {};
  private serverInstance: any = null;

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

  route(method: string, path: string, handler: any): this {
    const upperMethod = method.toUpperCase();
    if (!this.routes[path]) {
      this.routes[path] = {};
    }
    this.routes[path][upperMethod] = handler;
    return this;
  }

  launch(options?: ServeOptions): any {
    this.serverInstance = serve({
      routes: this.routes,
      ...options,
    });
    return this.serverInstance;
  }

  stop(): void {
    if (this.serverInstance) {
      this.serverInstance.stop();
    }
  }
}
