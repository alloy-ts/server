export * from "../dist/index.js";

import { serve, type ServeOptions, type Server } from "../dist/index.js";

/**
 * High-level Rocket application builder.
 */
export class App {
  private options?: ServeOptions;
  private routesTable: Record<string, any> = {};
  private activeServer?: Server;

  constructor(options?: ServeOptions) {
    this.options = options;
  }

  get(path: string, handler: any): this {
    this.routesTable[path] = { GET: handler };
    return this;
  }

  post(path: string, handler: any): this {
    this.routesTable[path] = { POST: handler };
    return this;
  }

  put(path: string, handler: any): this {
    this.routesTable[path] = { PUT: handler };
    return this;
  }

  delete(path: string, handler: any): this {
    this.routesTable[path] = { DELETE: handler };
    return this;
  }

  patch(path: string, handler: any): this {
    this.routesTable[path] = { PATCH: handler };
    return this;
  }

  mount(base: string, routes: Record<string, any>): this {
    for (const [pattern, handler] of Object.entries(routes)) {
      const fullPath = base === "/" ? pattern : `${base.replace(/\/$/, "")}${pattern}`;
      this.routesTable[fullPath] = handler;
    }
    return this;
  }

  launch(): Server {
    this.activeServer = serve(this.routesTable, undefined, this.options);
    return this.activeServer;
  }

  stop(): void {
    if (this.activeServer) {
      this.activeServer.stop();
    }
  }
}
