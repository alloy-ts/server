import * as native from "../dist/index.js";

export function main(): string {
  return "Hello, world!";
}

export const Response = native.Response;
export const file = native.file;
export const Catcher = native.Catcher;

function wrapHandler(handler: unknown): unknown {
  if (typeof handler === "function") {
    return (reqOrArgs: unknown) => {
      const req = Array.isArray(reqOrArgs) ? reqOrArgs[0] : reqOrArgs;
      return (handler as Function)(req);
    };
  }
  if (
    typeof handler === "object" &&
    handler !== null &&
    !(handler instanceof native.Response) &&
    !("path" in handler)
  ) {
    const wrappedMap: Record<string, unknown> = {};
    for (const [method, fn] of Object.entries(handler)) {
      wrappedMap[method] = wrapHandler(fn);
    }
    return wrappedMap;
  }
  return handler;
}

export const serve = (options: {
  routes?: Record<string, unknown>;
  fetch?: (req: native.RequestInput) => unknown;
  port?: number;
  hostname?: string;
  unix?: string;
}) => {
  const { routes, fetch, ...serveOptions } = options;

  let wrappedRoutes: Record<string, unknown> | undefined;
  if (routes) {
    wrappedRoutes = {};
    for (const [pattern, handler] of Object.entries(routes)) {
      wrappedRoutes[pattern] = wrapHandler(handler);
    }
  }

  const wrappedFetch = fetch ? wrapHandler(fetch) : undefined;

  return native.serve(wrappedRoutes, wrappedFetch, serveOptions);
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

export const Rocket = native.Rocket;

export const parseRouteAttribute = native.parseRouteAttribute;
export const parseCatcherAttribute = native.parseCatcherAttribute;
export const generateTsRouteManifest = native.generateTsRouteManifest;

export const Alloy = {
  serve,
  file,
  Response,
  Rocket,
  Catcher,
  uri,
  parseRouteAttribute,
  parseCatcherAttribute,
  generateTsRouteManifest,
};

export default Alloy;
