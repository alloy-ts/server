import { file, launch, registerRoute, Response, serve, Server } from "../index.js";

export { file, launch, registerRoute, Response, serve, Server };

export function main() {
  return "Hello, world!";
}

/**
 * Codegen / Decorator support for Rocket routes in TypeScript.
 */
export function get(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("GET", path, descriptor.value);
  };
}

export function post(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("POST", path, descriptor.value);
  };
}

export function put(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("PUT", path, descriptor.value);
  };
}

export function del(path: string) {
  return function (_target: object, _propertyKey: string, descriptor: PropertyDescriptor) {
    registerRoute("DELETE", path, descriptor.value);
  };
}

/**
 * Type-safe URI builder helper, mirroring Rocket's `uri!` macro for codegen.
 */
export function uri(path: string, params: Record<string, string | number> = {}): string {
  let result = path;
  for (const [key, val] of Object.entries(params)) {
    result = result.replace(`:${key}`, encodeURIComponent(String(val)));
    result = result.replace(`<${key}>`, encodeURIComponent(String(val)));
  }
  return result;
}

/**
 * High-performance Alloy server facade powered by Rocket NAPI bindings.
 */
export const Alloy = {
  serve,
  file,
  Response,
};
