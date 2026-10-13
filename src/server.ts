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

export const file = native.file;
export const registerRoute = native.registerRoute;
export const Server = native.Server;

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
