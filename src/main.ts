import { file, serve as nativeServe, type ResponseInit, type Server } from "../dist/index.js";
import { Response } from "../dist/index.js";

export * from "../dist/index.js";
export * from "./codegen.js";

/**
 * Helper to construct a Response instance matching Rocket's responder behavior.
 */
export function response(
  body?: string | null,
  initOrStatus?: number | ResponseInit,
  headers?: Array<[string, string]>,
): Response {
  if (typeof initOrStatus === "number") {
    return new Response(body, {
      status: initOrStatus,
      headers,
    });
  }
  return new Response(body, initOrStatus);
}

/**
 * `App` re-expresses `rocket::build()` and route registration at runtime for TypeScript.
 */
export class App {
  private routeTable: Record<string, Record<string, any>> = {};
  private activeServer: Server | null = null;

  private addRoute(method: string, path: string, handler: any) {
    const normalizedPath = path.startsWith("/") ? path : "/" + path;
    if (!this.routeTable[normalizedPath]) {
      this.routeTable[normalizedPath] = {};
    }
    this.routeTable[normalizedPath][method.toUpperCase()] = handler;
  }

  get(path: string, handler: any): this {
    this.addRoute("GET", path, handler);
    return this;
  }

  post(path: string, handler: any): this {
    this.addRoute("POST", path, handler);
    return this;
  }

  put(path: string, handler: any): this {
    this.addRoute("PUT", path, handler);
    return this;
  }

  delete(path: string, handler: any): this {
    this.addRoute("DELETE", path, handler);
    return this;
  }

  patch(path: string, handler: any): this {
    this.addRoute("PATCH", path, handler);
    return this;
  }

  mount(base: string, routes: Record<string, any>): this {
    const prefix = base.replace(/\/$/, "");
    for (const [p, handler] of Object.entries(routes)) {
      const fullPath = prefix + (p.startsWith("/") ? p : "/" + p);
      if (typeof handler === "function" || handler instanceof Response) {
        this.addRoute("GET", fullPath, handler);
      } else if (typeof handler === "object") {
        for (const [m, h] of Object.entries(handler)) {
          this.addRoute(m.toUpperCase(), fullPath, h);
        }
      }
    }
    return this;
  }

  launch(options: any = {}): Server {
    this.activeServer = serve({
      ...options,
      routes: {
        ...this.routeTable,
        ...options.routes,
      },
    });
    return this.activeServer;
  }

  stop(): void {
    if (this.activeServer) {
      this.activeServer.stop();
      this.activeServer = null;
    }
  }
}

export function serve(options: any = {}): Server {
  const envPort = process.env["BUN_PORT"] || process.env["PORT"] || process.env["NODE_PORT"];
  const port = options.port !== undefined ? options.port : envPort ? parseInt(envPort, 10) : 3000;
  const hostname = options.hostname || "0.0.0.0";

  return nativeServe({
    ...options,
    port,
    hostname,
  });
}

export const Alloy = {
  serve,
  file,
  App,
  response,
};

export default Alloy;
