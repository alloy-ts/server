import * as native from "../dist/index.js";

/**
 * Return default hello message.
 */
export function main(): string {
  return "Hello, world!";
}

// Re-export all NAPI native exports
export * from "../dist/index.js";

// Additional TypeScript abstractions for Rocket Codegen

export interface RouteDecoratorOptions {
  rank?: number;
  format?: string;
  data?: string;
}

/**
 * Route metadata stored on target handlers.
 */
export interface RegisteredRoute {
  method: string;
  path: string;
  handler: Function;
  meta: native.RouteMeta;
}

const REGISTERED_ROUTES: RegisteredRoute[] = [];

/**
 * Helper to register route decorator metadata.
 */
function createRouteDecorator(method: string) {
  return function (path: string, options?: RouteDecoratorOptions) {
    return function (target: any, _propertyKey: string, descriptor: PropertyDescriptor) {
      let attr = path;
      if (options) {
        const parts: string[] = [`"${path}"`];
        if (options.rank !== undefined) parts.push(`rank = ${options.rank}`);
        if (options.format !== undefined) parts.push(`format = "${options.format}"`);
        if (options.data !== undefined) parts.push(`data = "${options.data}"`);
        attr = parts.join(", ");
      }
      const meta = native.parseRouteAttribute(method, attr);
      const handler = descriptor ? descriptor.value : target;
      REGISTERED_ROUTES.push({
        method,
        path: meta.path,
        handler,
        meta,
      });
      native.registerRoute(method, meta.path, handler);
    };
  };
}

export const Get = createRouteDecorator("GET");
export const Post = createRouteDecorator("POST");
export const Put = createRouteDecorator("PUT");
export const Delete = createRouteDecorator("DELETE");
export const Head = createRouteDecorator("HEAD");
export const Options = createRouteDecorator("OPTIONS");
export const Patch = createRouteDecorator("PATCH");

export function RouteDecorator(method: string, path: string, options?: RouteDecoratorOptions) {
  return createRouteDecorator(method)(path, options);
}

export function Catch(code?: number | "default") {
  return function (_target: any, _propertyKey: string, _descriptor: PropertyDescriptor) {
    const attr = code === undefined ? "default" : String(code);
    const meta = native.parseCatchAttribute(attr);
    return meta;
  };
}

/**
 * Type-safe URI builder.
 */
export function uri(
  pattern: string,
  params: Record<string, any> = {},
  prefix?: string,
  suffix?: string,
): string {
  const paramStrings: Record<string, string> = {};
  for (const [k, v] of Object.entries(params)) {
    if (v !== undefined && v !== null) {
      paramStrings[k] = String(v);
    }
  }
  return native.buildUri(pattern, paramStrings, prefix, suffix);
}

/**
 * Helper macro function `routes![...]` equivalent for TS.
 */
export function routes(...handlers: Function[]): RegisteredRoute[] {
  return REGISTERED_ROUTES.filter((r) => handlers.includes(r.handler));
}

/**
 * Helper macro function `catchers![...]` equivalent for TS.
 */
export function catchers(...catchers: any[]): any[] {
  return catchers;
}

/**
 * Rocket Codegen namespace providing codegen utilities.
 */
export const Codegen = {
  parseRoute: native.parseRouteAttribute,
  parseCatch: native.parseCatchAttribute,
  buildUri: native.buildUri,
  generateTsRoutes: native.generateTsRoutes,
  generateFormSchema: native.generateFormSchema,
};
