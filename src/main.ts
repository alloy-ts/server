import * as native from "../dist/index.js";

export * from "../dist/index.js";

export class Response {
  status: number;
  headers: Record<string, string>;
  body?: Array<number>;

  constructor(body?: string | Uint8Array | null, init?: native.ResponseInit | null) {
    const res = native.response(typeof body === "string" ? body : undefined, init ?? undefined);
    this.status = res.status;
    this.headers = res.headers;
    this.body = res.body;
    if (body instanceof Uint8Array) {
      this.body = Array.from(body);
    }
  }

  static json(data: unknown, init?: native.ResponseInit | null): native.Response {
    return native.responseJson(data, init);
  }

  static redirect(url: string, status?: number | null): native.Response {
    return native.responseRedirect(url, status);
  }
}

export function serve(
  routesOrOptions?: unknown,
  fetchFn?: unknown,
  options?: native.ServeOptions | null,
): native.Server {
  if (
    routesOrOptions &&
    typeof routesOrOptions === "object" &&
    ("routes" in routesOrOptions ||
      "fetch" in routesOrOptions ||
      "port" in routesOrOptions ||
      "hostname" in routesOrOptions ||
      "unix" in routesOrOptions)
  ) {
    const opts = routesOrOptions as {
      routes?: unknown;
      fetch?: unknown;
      port?: number;
      hostname?: string;
      unix?: string;
    };
    return native.serve(opts.routes, opts.fetch, {
      port: opts.port,
      hostname: opts.hostname,
      unix: opts.unix,
    });
  }
  return native.serve(routesOrOptions, fetchFn, options);
}

export const Alloy = {
  serve,
  file: native.file,
  Response,
};

export const Bun = Alloy;

export function main(): string {
  return "Hello, world!";
}
