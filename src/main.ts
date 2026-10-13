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

export class App {
  private routesTable: Record<string, unknown> = {};
  private serverInstance?: native.Server;

  get(path: string, handler: unknown): this {
    return this.route("GET", path, handler);
  }

  post(path: string, handler: unknown): this {
    return this.route("POST", path, handler);
  }

  put(path: string, handler: unknown): this {
    return this.route("PUT", path, handler);
  }

  delete(path: string, handler: unknown): this {
    return this.route("DELETE", path, handler);
  }

  patch(path: string, handler: unknown): this {
    return this.route("PATCH", path, handler);
  }

  head(path: string, handler: unknown): this {
    return this.route("HEAD", path, handler);
  }

  options(path: string, handler: unknown): this {
    return this.route("OPTIONS", path, handler);
  }

  route(method: string, path: string, handler: unknown): this {
    const m = method.toUpperCase();
    const existing = this.routesTable[path];
    if (existing && typeof existing === "object" && !(existing instanceof Response)) {
      (existing as Record<string, unknown>)[m] = handler;
    } else if (existing) {
      this.routesTable[path] = {
        GET: existing,
        [m]: handler,
      };
    } else {
      this.routesTable[path] = {
        [m]: handler,
      };
    }
    return this;
  }

  mount(base: string, routes: Record<string, unknown>): this {
    const prefix = base === "/" ? "" : base;
    for (const [path, val] of Object.entries(routes)) {
      const fullPath = `${prefix}${path.startsWith("/") ? "" : "/"}${path}`;
      this.routesTable[fullPath] = val;
    }
    return this;
  }

  launch(options?: native.ServeOptions | null): native.Server {
    this.serverInstance = serve({
      routes: this.routesTable,
      port: options?.port,
      hostname: options?.hostname,
      unix: options?.unix,
    });
    return this.serverInstance;
  }

  stop(): void {
    if (this.serverInstance) {
      this.serverInstance.stop();
    }
  }
}

export const Alloy = {
  serve,
  file: native.file,
  Response,
  App,
};

export const Bun = Alloy;

export function main(): string {
  return "Hello, world!";
}
