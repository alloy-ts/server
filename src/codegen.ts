//! TypeScript codegen layer for Rocket / Alloy web framework.

import {
  parseRoutePattern,
  buildRouteUri,
  parseCatcherCode,
  generateRouteManifest,
  registerRoute,
} from "../native/index.js";

export interface RouteMetadata {
  method: string;
  path: string;
  handler: Function;
  target?: any;
  propertyKey?: string | symbol;
}

export interface CatcherMetadata {
  code: number | null;
  handler: Function;
  target?: any;
  propertyKey?: string | symbol;
}

const registeredRoutes: RouteMetadata[] = [];
const registeredCatchers: CatcherMetadata[] = [];

function createRouteDecorator(method: string) {
  return function (path: string) {
    return function (targetOrFn: any, propertyKeyOrContext?: any, descriptor?: PropertyDescriptor) {
      let handler = targetOrFn;
      let target: any = targetOrFn;
      let propertyKey = propertyKeyOrContext;

      if (descriptor && descriptor.value) {
        handler = descriptor.value;
      } else if (
        typeof propertyKeyOrContext === "object" &&
        propertyKeyOrContext !== null &&
        "name" in propertyKeyOrContext
      ) {
        propertyKey = propertyKeyOrContext.name;
      }

      registeredRoutes.push({
        method,
        path,
        handler,
        target,
        propertyKey,
      });
      registerRoute(method, path, handler);
      return descriptor || targetOrFn;
    };
  };
}

export const get = createRouteDecorator("GET");
export const post = createRouteDecorator("POST");
export const put = createRouteDecorator("PUT");
export const del = createRouteDecorator("DELETE");
export { del as delete };
export const head = createRouteDecorator("HEAD");
export const options = createRouteDecorator("OPTIONS");
export const patch = createRouteDecorator("PATCH");

export function route(method: string, path: string) {
  return function (targetOrFn: any, propertyKeyOrContext?: any, descriptor?: PropertyDescriptor) {
    let handler = targetOrFn;
    let target: any = targetOrFn;
    let propertyKey = propertyKeyOrContext;

    if (descriptor && descriptor.value) {
      handler = descriptor.value;
    } else if (
      typeof propertyKeyOrContext === "object" &&
      propertyKeyOrContext !== null &&
      "name" in propertyKeyOrContext
    ) {
      propertyKey = propertyKeyOrContext.name;
    }

    registeredRoutes.push({
      method: method.toUpperCase(),
      path,
      handler,
      target,
      propertyKey,
    });
    registerRoute(method.toUpperCase(), path, handler);
    return descriptor || targetOrFn;
  };
}

export function catchCatcher(status: number | string) {
  const code = typeof status === "number" ? status : (parseCatcherCode(status) ?? null);
  return function (targetOrFn: any, propertyKeyOrContext?: any, descriptor?: PropertyDescriptor) {
    let handler = targetOrFn;
    let target: any = targetOrFn;
    let propertyKey = propertyKeyOrContext;

    if (descriptor && descriptor.value) {
      handler = descriptor.value;
    } else if (
      typeof propertyKeyOrContext === "object" &&
      propertyKeyOrContext !== null &&
      "name" in propertyKeyOrContext
    ) {
      propertyKey = propertyKeyOrContext.name;
    }

    registeredCatchers.push({
      code,
      handler,
      target,
      propertyKey,
    });
    return descriptor || targetOrFn;
  };
}

export { catchCatcher as catch };

export function FromForm(_options?: { validate?: (data: any) => boolean }) {
  return function (target: any) {
    return target;
  };
}

export function Responder(_options?: { status?: number; contentType?: string }) {
  return function (target: any) {
    return target;
  };
}

export function parseFormData(body: string | Buffer): Record<string, string> {
  const str = typeof body === "string" ? body : body.toString("utf-8");
  const params = new URLSearchParams(str);
  const result: Record<string, string> = {};
  for (const [key, value] of params.entries()) {
    result[key] = value;
  }
  return result;
}

export function routes(items?: Function[]): RouteMetadata[] {
  if (items && items.length > 0) {
    return registeredRoutes.filter((r) => items.includes(r.handler));
  }
  return [...registeredRoutes];
}

export function catchers(items?: Function[]): CatcherMetadata[] {
  if (items && items.length > 0) {
    return registeredCatchers.filter((c) => items.includes(c.handler));
  }
  return [...registeredCatchers];
}

export function uri(template: string, params?: Record<string, any>): string {
  if (!params) {
    return buildRouteUri(template, undefined);
  }
  const strParams: Record<string, string> = {};
  for (const [key, val] of Object.entries(params)) {
    if (val !== undefined && val !== null) {
      strParams[key] = String(val);
    }
  }
  return buildRouteUri(template, strParams);
}

export { parseRoutePattern, buildRouteUri, parseCatcherCode, generateRouteManifest };
