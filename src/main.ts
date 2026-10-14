import * as native from "../dist/index.js";
import type { ServeOptions, Server } from "../dist/index.js";

export * from "../dist/index.js";

export class App {
  private routes: Record<string, any> = {};
  private serverOptions: ServeOptions;
  private serverHandle: Server | null = null;

  constructor(options: ServeOptions = {}) {
    this.serverOptions = options;
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

  options(path: string, handler: any): this {
    return this.route("OPTIONS", path, handler);
  }

  route(method: string, path: string, handler: any): this {
    const m = method.toUpperCase();
    if (!this.routes[path]) {
      this.routes[path] = {};
    }
    if (typeof this.routes[path] === "object" && !("status" in this.routes[path])) {
      this.routes[path][m] = handler;
    } else {
      const existing = this.routes[path];
      this.routes[path] = { GET: existing, [m]: handler };
    }
    return this;
  }

  launch(): Server {
    this.serverHandle = native.serve(this.routes, null, this.serverOptions);
    return this.serverHandle;
  }

  stop(): void {
    if (this.serverHandle) {
      this.serverHandle.stop();
    }
  }
}

export const Alloy = {
  serve: native.serve,
  file: native.file,
  response: native.response,
  responseJson: native.responseJson,
  responseRedirect: native.responseRedirect,
  registerRoute: native.registerRoute,
  Config: native.Config,
  Rocket: native.Rocket,
  Route: native.Route,
  Handler: native.Handler,
  createHandlerOutcomeSuccess: native.createHandlerOutcomeSuccess,
  createHandlerOutcomeError: native.createHandlerOutcomeError,
  createHandlerOutcomeForward: native.createHandlerOutcomeForward,
  launch: native.launch,
  rocketMain: native.rocketMain,
  App,
};

export default Alloy;
