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

export const parseRouteAttribute = native.parseRouteAttribute;
export const parseCatcherAttribute = native.parseCatcherAttribute;
export const buildUri = native.buildUri;
export const generateTsRouteManifest = native.generateTsRouteManifest;

export const Rocket = native.Rocket;
export const Config = native.Config;

export const Alloy = {
  serve,
  file,
  Response,
  Rocket,
  Config,
  uri,
  parseRouteAttribute,
  parseCatcherAttribute,
  buildUri,
  generateTsRouteManifest,
};

export default Alloy;
