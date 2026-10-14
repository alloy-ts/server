import * as native from "../index.js";

export interface CodegenParam {
  name: string;
  kind: string;
  tsType: string;
}

export interface CodegenRouteSpec {
  method: string;
  uri: string;
  rank?: number;
  format?: string;
  data?: string;
  name?: string;
}

export interface CodegenRoute {
  name: string;
  method: string;
  path: string;
  rocketPath: string;
  rank?: number;
  format?: string;
  data?: string;
  params: CodegenParam[];
  macroIdent: string;
}

export interface CodegenCatcher {
  name: string;
  code?: number;
  isDefault: boolean;
}

export interface CodegenUri {
  uri: string;
  path: string;
  query?: string;
  params: Record<string, string>;
}

export interface ShutdownOptions {
  grace?: number;
  mercy?: number;
  force?: number;
}

export interface RocketConfig {
  address?: string;
  port?: number;
  workers?: number;
  maxBlocking?: number;
  keepAlive?: number;
  ident?: string;
  logLevel?: string;
  shutdown?: ShutdownOptions;
}

export interface Limits {
  limits: Record<string, string>;
}

export interface Shutdown {
  ctrlc?: boolean;
  signals?: string[];
  grace?: number;
  mercy?: number;
  force?: number;
}

export interface ConfigInit {
  profile?: string;
  address?: string;
  port?: number;
  workers?: number;
  maxBlocking?: number;
  ident?: string;
  identNone?: boolean;
  ipHeader?: string;
  ipHeaderNone?: boolean;
  limits?: Limits;
  tempDir?: string;
  keepAlive?: number;
  shutdown?: Shutdown;
  logLevel?: string;
  cliColors?: boolean;
}

export function main(): string {
  return "Hello, world!";
}

function wrapHandler(fn: any) {
  if (typeof fn === "function") {
    return (err: any, req: any) => {
      let payload = req;
      if (Array.isArray(req)) {
        payload = req[0];
      } else if (Array.isArray(err)) {
        payload = err[0];
      } else if (!payload && err && typeof err === "object" && "path" in err) {
        payload = err;
      }
      return fn(payload);
    };
  }
  if (fn && typeof fn === "object" && !(fn instanceof native.Response) && !("path" in fn)) {
    const wrapped: Record<string, any> = {};
    for (const [key, val] of Object.entries(fn)) {
      wrapped[key] = wrapHandler(val);
    }
    return wrapped;
  }
  return fn;
}

export function serve(options: any) {
  const opts = { ...options };
  if (opts.routes) {
    const wrappedRoutes: Record<string, any> = {};
    for (const [pattern, val] of Object.entries(opts.routes)) {
      wrappedRoutes[pattern] = wrapHandler(val);
    }
    opts.routes = wrappedRoutes;
  }
  if (typeof opts.fetch === "function") {
    opts.fetch = wrapHandler(opts.fetch);
  }
  return native.serve(opts);
}

export const Response = native.Response;
export const file = native.file;
export const Config = native.Config;
export const Rocket = native.Rocket;
export const Route = native.Route;
export const launch = native.launch;
export const rocketMain = native.rocketMain;

export function response(body: string | Buffer | object, status = 200) {
  if (typeof body === "object" && !(body instanceof Buffer)) {
    return Response.json(body, { status });
  }
  return new Response(typeof body === "string" ? body : (body as Buffer).toString("utf-8"), {
    status,
  });
}

export class App {
  private rocket: any;

  constructor(config?: RocketConfig) {
    this.rocket = new Rocket(config ?? null);
  }

  public get(path: string, handler: any): this {
    return this.route("GET", path, handler);
  }

  public post(path: string, handler: any): this {
    return this.route("POST", path, handler);
  }

  public put(path: string, handler: any): this {
    return this.route("PUT", path, handler);
  }

  public delete(path: string, handler: any): this {
    return this.route("DELETE", path, handler);
  }

  public route(method: string, path: string, handler: any): this {
    const wrapped =
      typeof handler === "function" ? (err: any, req: any) => handler(req || err) : handler;
    this.rocket.mount("/", method, path, wrapped);
    return this;
  }

  public catch(code: number | undefined, handler: any): this {
    const wrapped =
      typeof handler === "function" ? (err: any, req: any) => handler(req || err) : handler;
    this.rocket.register("/", code ?? null, wrapped);
    return this;
  }

  public async launch(): Promise<void> {
    return this.rocket.launch();
  }

  public shutdown(): void {
    return this.rocket.shutdown();
  }
}

export const generateRoute = native.generateRoute;
export const generateRoutes = native.generateRoutes;
export const generateCatcher = native.generateCatcher;
export const generateUri = native.generateUri;
export const generateTsDeclarations = native.generateTsDeclarations;

// TypeScript Codegen Helper Macros / Decorators matching Rocket API
export function route(method: string, uri: string, spec?: Partial<CodegenRouteSpec>): CodegenRoute {
  return native.generateRoute({
    method,
    uri,
    rank: spec?.rank,
    format: spec?.format,
    data: spec?.data,
    name: spec?.name,
  });
}

export function get(uri: string, spec?: Partial<CodegenRouteSpec>): CodegenRoute {
  return route("GET", uri, spec);
}

export function post(uri: string, spec?: Partial<CodegenRouteSpec>): CodegenRoute {
  return route("POST", uri, spec);
}

export function put(uri: string, spec?: Partial<CodegenRouteSpec>): CodegenRoute {
  return route("PUT", uri, spec);
}

export function del(uri: string, spec?: Partial<CodegenRouteSpec>): CodegenRoute {
  return route("DELETE", uri, spec);
}

export function catchStatus(code?: number, name?: string): CodegenCatcher {
  return native.generateCatcher(code, name);
}

export function routes(specs: CodegenRouteSpec[]): CodegenRoute[] {
  return native.generateRoutes(specs);
}

export function catchers(specs: Array<{ code?: number; name?: string }>): CodegenCatcher[] {
  return specs.map((s) => native.generateCatcher(s.code, s.name));
}

export function uri(
  routePath: string,
  params?: Record<string, string>,
  prefix?: string,
  suffix?: string,
): CodegenUri {
  return native.generateUri(routePath, params, prefix, suffix);
}

export const codegen = {
  generateRoute,
  generateRoutes,
  generateCatcher,
  generateUri,
  generateTsDeclarations,
  route,
  get,
  post,
  put,
  delete: del,
  catch: catchStatus,
  routes,
  catchers,
  uri,
};

export const Alloy = {
  serve,
  file,
  Response,
  response,
  Config,
  Rocket,
  Route,
  App,
  launch,
  rocketMain,
  codegen,
};

export default Alloy;
