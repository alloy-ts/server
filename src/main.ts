import * as native from "../dist/index.js";

export function main(): string {
  return "Hello, world!";
}

export const Alloy = {
  serve(options: {
    routes?: unknown;
    fetch?: unknown;
    port?: number;
    hostname?: string;
    unix?: string;
  }) {
    const { routes, fetch, ...opts } = options;
    return native.serve(routes, fetch, opts);
  },
  file: native.file,
};

export const serve = native.serve;
export const file = native.file;
export const response = native.response;
export const responseJson = native.responseJson;
export const responseRedirect = native.responseRedirect;
export const registerRoute = native.registerRoute;
export const Codegen = native.Codegen;
export const parseRoutePattern = native.parseRoutePattern;
export const Config = native.Config;

export { native };
