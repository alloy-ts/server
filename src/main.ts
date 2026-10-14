import {
  serve as nativeServe,
  file as nativeFile,
  Response as NativeResponse,
  Config as NativeConfig,
  Server,
  launch,
  registerRoute,
} from "../native/index.js";

import * as codegen from "./codegen.ts";

export interface ServeOptions {
  port?: number;
  hostname?: string;
  unix?: string;
  routes?: Record<string, any>;
  fetch?: (req: any) => any;
}

export function serve(options: ServeOptions): Server {
  return nativeServe(options);
}

export function file(path: string) {
  return nativeFile(path);
}

export class Response extends NativeResponse {
  static override json(
    data: any,
    init?: { status?: number; headers?: [string, string][] },
  ): Response {
    return NativeResponse.json(data, init);
  }

  static override redirect(url: string, status?: number): Response {
    return NativeResponse.redirect(url, status);
  }
}

export function response(
  body?: string | Buffer,
  init?: { status?: number; headers?: [string, string][] },
): Response {
  const bodyStr = typeof body === "string" ? body : body ? body.toString("utf-8") : undefined;
  return new Response(bodyStr, init);
}

export type Middleware = (
  req: any,
  next: () => Promise<Response> | Response,
) => Promise<Response> | Response;

export class App {
  private options: ServeOptions = {
    port: 3000,
    hostname: "0.0.0.0",
    routes: {},
  };
  private middlewares: Middleware[] = [];
  private serverInstance?: Server;

  constructor(options?: ServeOptions) {
    if (options) {
      this.options = { ...this.options, ...options };
    }
  }

  use(middleware: Middleware): this {
    this.middlewares.push(middleware);
    return this;
  }

  attach(fairing: Middleware): this {
    return this.use(fairing);
  }

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

  optionsRoute(path: string, handler: any): this {
    return this.route("OPTIONS", path, handler);
  }

  route(method: string, path: string, handler: any): this {
    if (!this.options.routes) {
      this.options.routes = {};
    }

    const wrappedHandler = async (req: any) => {
      let idx = 0;
      const next = async (): Promise<Response> => {
        if (idx < this.middlewares.length) {
          const mw = this.middlewares[idx++];
          if (mw) {
            return mw(req, next);
          }
        }
        return handler(req);
      };
      return next();
    };

    const current = this.options.routes[path];
    if (typeof current === "object" && current !== null && !(current instanceof Response)) {
      current[method.toUpperCase()] = wrappedHandler;
    } else {
      this.options.routes[path] = {
        [method.toUpperCase()]: wrappedHandler,
      };
    }
    return this;
  }

  mount(base: string, routesMap: Record<string, any>): this {
    if (!this.options.routes) {
      this.options.routes = {};
    }
    for (const [p, val] of Object.entries(routesMap)) {
      const fullPath = base === "/" ? p : `${base.replace(/\/$/, "")}/${p.replace(/^\//, "")}`;
      this.options.routes[fullPath] = val;
    }
    return this;
  }

  launch(options?: ServeOptions): Server {
    if (options) {
      this.options = { ...this.options, ...options };
    }
    this.serverInstance = serve(this.options);
    return this.serverInstance;
  }

  stop(): void {
    if (this.serverInstance) {
      this.serverInstance.stop();
    }
  }
}

export const Config = NativeConfig;

export const Alloy = {
  serve,
  file,
  Response,
  Config,
  App,
  response,
};

export { Server, launch, registerRoute, codegen };

export * from "./codegen.ts";

export function main(): string {
  return "Hello, world!";
}

export default Alloy;
