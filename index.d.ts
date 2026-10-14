export * from "./dist/index.d.ts";

import { Server, Response, Catcher } from "./dist/index.d.ts";

export interface AppOptions {
  port?: number;
  hostname?: string;
}

export class App {
  constructor(options?: AppOptions);
  get(path: string, handler: any): this;
  post(path: string, handler: any): this;
  put(path: string, handler: any): this;
  delete(path: string, handler: any): this;
  patch(path: string, handler: any): this;
  route(method: string, path: string, handler: any): this;
  mount(basePath: string, routes: Record<string, any>): this;
  launch(): Server;
  stop(): void;
}

export function uri(pattern: string, params?: Record<string, string>): string;

export function get(path: string): MethodDecorator;
export function post(path: string): MethodDecorator;
export function put(path: string): MethodDecorator;
export function del(path: string): MethodDecorator;
export function route(method: string, path: string): MethodDecorator;
export function catchCatcher(code: number | string): MethodDecorator;

export interface RegisteredRoute {
  method: string;
  path: string;
  handler: any;
}

export interface RegisteredCatcher {
  code: number | string;
  handler: any;
}

export function routes(): RegisteredRoute[];
export function catchers(): RegisteredCatcher[];

declare const Alloy: {
  main: () => string;
  serve: (options: any) => Server;
  file: (path: string) => any;
  response: (body?: string | null, options?: any) => Response;
  Response: typeof Response;
  Config: any;
  Catcher: typeof Catcher;
  App: typeof App;
  uri: typeof uri;
  routes: typeof routes;
  catchers: typeof catchers;
  get: typeof get;
  post: typeof post;
  put: typeof put;
  route: typeof route;
  catchCatcher: typeof catchCatcher;
};

export default Alloy;
