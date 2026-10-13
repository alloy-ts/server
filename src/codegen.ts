import {
  generateTsRouteManifest,
  generateUri,
  parseCatchAttribute,
  parseRouteAttribute,
  type ParsedCatch,
  type ParsedRoute,
  type RouteMetadataInput,
} from "../index.js";

export {
  generateTsRouteManifest,
  generateUri,
  parseCatchAttribute,
  parseRouteAttribute,
  type ParsedCatch,
  type ParsedRoute,
  type RouteMetadataInput,
};

export interface RouteOptions {
  rank?: number;
  format?: string;
  data?: string;
}

export interface RouteMetadata {
  method: string;
  uri: string;
  handler: Function;
  options?: RouteOptions;
}

export interface CatcherMetadata {
  code?: number;
  isDefault: boolean;
  handler: Function;
}

export function createRoute(
  method: string,
  uri: string,
  handler: Function,
  options?: RouteOptions,
): RouteMetadata {
  return {
    method: method.toUpperCase(),
    uri,
    handler,
    options,
  };
}

export function Get(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("GET", uri, handler, options);
}

export function Post(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("POST", uri, handler, options);
}

export function Put(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("PUT", uri, handler, options);
}

export function Delete(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("DELETE", uri, handler, options);
}

export function Head(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("HEAD", uri, handler, options);
}

export function Options(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("OPTIONS", uri, handler, options);
}

export function Patch(uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute("PATCH", uri, handler, options);
}

export function Route(method: string, uri: string, options?: RouteOptions) {
  return (handler: Function): RouteMetadata => createRoute(method, uri, handler, options);
}

export function Catch(status: number | "default") {
  return (handler: Function): CatcherMetadata => ({
    code: typeof status === "number" ? status : undefined,
    isDefault: status === "default",
    handler,
  });
}

/**
 * Collects route definitions into a route table mapping URI -> Method -> Handler.
 */
export function routes(
  ...routeInputs: Array<RouteMetadata | RouteMetadata[] | Record<string, Function>>
): Record<string, Record<string, Function>> {
  const table: Record<string, Record<string, Function>> = {};

  const processRoute = (r: RouteMetadata) => {
    let subTable = table[r.uri];
    if (!subTable) {
      subTable = {};
      table[r.uri] = subTable;
    }
    subTable[r.method] = r.handler;
  };

  for (const input of routeInputs) {
    if (Array.isArray(input)) {
      for (const r of input) {
        processRoute(r);
      }
    } else if (
      input &&
      typeof input === "object" &&
      "method" in input &&
      "uri" in input &&
      "handler" in input
    ) {
      processRoute(input as RouteMetadata);
    } else if (input && typeof input === "object") {
      for (const [uriPattern, value] of Object.entries(input)) {
        if (typeof value === "function") {
          let subTable = table[uriPattern];
          if (!subTable) {
            subTable = {};
            table[uriPattern] = subTable;
          }
          subTable["GET"] = value;
        } else if (typeof value === "object" && value !== null) {
          for (const [method, handler] of Object.entries(value)) {
            if (typeof handler === "function") {
              let subTable = table[uriPattern];
              if (!subTable) {
                subTable = {};
                table[uriPattern] = subTable;
              }
              subTable[method.toUpperCase()] = handler;
            }
          }
        }
      }
    }
  }

  return table;
}

/**
 * Collects catchers from catcher metadata objects.
 */
export function catchers(
  ...catcherInputs: Array<CatcherMetadata | CatcherMetadata[]>
): CatcherMetadata[] {
  const result: CatcherMetadata[] = [];

  for (const input of catcherInputs) {
    if (Array.isArray(input)) {
      result.push(...input);
    } else if (input && typeof input === "object" && "handler" in input) {
      result.push(input);
    }
  }

  return result;
}

/**
 * Type-safe Rocket URI generator.
 */
export function uri(template: string, params?: Record<string, string>): string {
  return generateUri(template, params);
}

/**
 * Form parsing helper (`FromForm`).
 */
export class FromForm {
  static parse<T extends object>(body: string | Record<string, unknown>): T {
    if (typeof body === "object" && body !== null) {
      return body as T;
    }
    const params = new URLSearchParams(body);
    const result: Record<string, unknown> = {};
    for (const [key, value] of params.entries()) {
      result[key] = value;
    }
    return result as T;
  }
}

/**
 * Responder helper for converting values into Rocket responses.
 */
export class Responder {
  static format(
    val: unknown,
    status = 200,
    headers?: Record<string, string>,
  ): { status: number; headers: Record<string, string>; body?: Buffer } {
    if (typeof val === "string") {
      return {
        status,
        headers: { "Content-Type": "text/plain", ...headers },
        body: Buffer.from(val),
      };
    }
    if (val && typeof val === "object" && !("body" in val)) {
      return {
        status,
        headers: { "Content-Type": "application/json", ...headers },
        body: Buffer.from(JSON.stringify(val)),
      };
    }
    return (
      (val as { status: number; headers: Record<string, string>; body?: Buffer }) || {
        status,
        headers: headers || {},
      }
    );
  }
}
