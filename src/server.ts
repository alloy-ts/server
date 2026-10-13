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
      routesOrOptions.hostname !== undefined)
  ) {
    const { routes, fetch, ...scalarOpts } = routesOrOptions;
    return native.serve(routes, fetch, scalarOpts);
  }
  return native.serve(routesOrOptions, fetchFn, opts);
}
