import {
  buildRouteUri as nativeBuildRouteUri,
  generateTsRouteTypes as nativeGenerateTsRouteTypes,
  parseRoutePattern as nativeParseRoutePattern,
  registerRoute,
  type RouteInfo,
} from "../dist/index.js";

export {
  nativeBuildRouteUri as buildRouteUri,
  nativeGenerateTsRouteTypes as generateTsRouteTypes,
  nativeParseRoutePattern as parseRoutePattern,
};

/**
 * Type-safe URI builder corresponding to Rocket's `uri!` macro.
 *
 * Example:
 * ```ts
 * const userUrl = uri("/users/:id", { id: "123" });
 * // "/users/123"
 * ```
 */
export function uri(pattern: string, params: Record<string, string> = {}): string {
  return nativeBuildRouteUri(pattern, params);
}

export interface RouteMeta {
  method: string;
  path: string;
  handler: (req: any) => any;
}

const registeredRoutesMeta: RouteMeta[] = [];

export function getRegisteredRoutesMeta(): RouteMeta[] {
  return [...registeredRoutesMeta];
}

export function route(method: string, path: string, handler: (req: any) => any): void {
  registeredRoutesMeta.push({ method, path, handler });
  registerRoute(method, path, handler);
}

export function get(path: string, handler: (req: any) => any): void {
  route("GET", path, handler);
}

export function post(path: string, handler: (req: any) => any): void {
  route("POST", path, handler);
}

export function put(path: string, handler: (req: any) => any): void {
  route("PUT", path, handler);
}

export function del(path: string, handler: (req: any) => any): void {
  route("DELETE", path, handler);
}

/**
 * Codegen helper: generate TypeScript definition interfaces for registered route patterns.
 */
export function generateRoutesTypeDefinitions(patterns: string[]): string {
  const routesInfo: RouteInfo[] = patterns.map((p) => nativeParseRoutePattern(p));
  return nativeGenerateTsRouteTypes(routesInfo);
}
