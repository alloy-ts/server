import * as native from "../dist/index.js";

export function main(): string {
  return "Hello, world!";
}

export const response = native.response;
export const responseJson = native.responseJson;
export const responseRedirect = native.responseRedirect;

export const Response = Object.assign(native.response, {
  json: native.responseJson,
  redirect: native.responseRedirect,
});

export const file = native.file;
export const serve = (options: {
  routes?: Record<string, unknown>;
  fetch?: (req: native.RequestInput) => unknown;
  port?: number;
  hostname?: string;
  unix?: string;
}) => {
  const { routes, fetch, ...serveOptions } = options;
  return native.serve(routes, fetch, serveOptions);
};

export function uri(
  template: string,
  params: Record<string, string | number | boolean> = {},
): string {
  const strParams: Record<string, string> = {};
  for (const [k, v] of Object.entries(params)) {
    strParams[k] = String(v);
  }
  return native.buildUri(template, strParams);
}

export class App {
  private routesTable: Record<string, unknown> = {};
  private options: { port?: number; hostname?: string; unix?: string };

  constructor(options: { port?: number; hostname?: string; unix?: string } = {}) {
    this.options = options;
  }

  get(path: string, handler: unknown): this {
    this.registerMethodRoute(path, "GET", handler);
    return this;
  }

  post(path: string, handler: unknown): this {
    this.registerMethodRoute(path, "POST", handler);
    return this;
  }

  put(path: string, handler: unknown): this {
    this.registerMethodRoute(path, "PUT", handler);
    return this;
  }

  delete(path: string, handler: unknown): this {
    this.registerMethodRoute(path, "DELETE", handler);
    return this;
  }

  patch(path: string, handler: unknown): this {
    this.registerMethodRoute(path, "PATCH", handler);
    return this;
  }

  private registerMethodRoute(path: string, method: string, handler: unknown): void {
    if (typeof this.routesTable[path] === "object" && this.routesTable[path] !== null) {
      (this.routesTable[path] as Record<string, unknown>)[method] = handler;
    } else if (typeof this.routesTable[path] === "undefined") {
      this.routesTable[path] = { [method]: handler };
    } else {
      const existing = this.routesTable[path];
      this.routesTable[path] = { GET: existing, [method]: handler };
    }
  }

  launch(): native.Server {
    return serve({ routes: this.routesTable, ...this.options });
  }
}

export const parseRouteAttribute = native.parseRouteAttribute;
export const parseCatcherAttribute = native.parseCatcherAttribute;
export const buildUri = native.buildUri;
export const generateTsRouteManifest = native.generateTsRouteManifest;
export const createRoute = native.createRoute;
export const createHandler = native.createHandler;
export const HandlerOutcomeStatus = native.HandlerOutcomeStatus;

export const Rocket = native.Rocket;
export const Config = native.Config;

export const Alloy = {
  serve,
  file,
  Response,
  Rocket,
  Config,
  uri,
  App,
  parseRouteAttribute,
  parseCatcherAttribute,
  buildUri,
  generateTsRouteManifest,
  createRoute,
  createHandler,
  HandlerOutcomeStatus,
};

export default Alloy;
