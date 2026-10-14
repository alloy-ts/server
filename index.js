import * as native from "./dist/index.js";

export * from "./dist/index.js";

const _registeredRoutes = [];
const _registeredCatchers = [];

/**
 * High-level App class corresponding to rocket::build().
 */
export class App {
  constructor(options = {}) {
    this.options = options;
    this.routesTable = {};
  }

  get(path, handler) {
    return this.route("GET", path, handler);
  }

  post(path, handler) {
    return this.route("POST", path, handler);
  }

  put(path, handler) {
    return this.route("PUT", path, handler);
  }

  delete(path, handler) {
    return this.route("DELETE", path, handler);
  }

  patch(path, handler) {
    return this.route("PATCH", path, handler);
  }

  route(method, path, handler) {
    if (!this.routesTable[path]) {
      this.routesTable[path] = {};
    }
    this.routesTable[path][method.toUpperCase()] = handler;
    native.registerRoute(method, path, handler);
    return this;
  }

  mount(basePath, routes) {
    for (const [pattern, handler] of Object.entries(routes)) {
      const fullPath = basePath === "/" ? pattern : `${basePath}${pattern}`;
      this.routesTable[fullPath] = handler;
    }
    return this;
  }

  launch() {
    this.server = native.serve({
      port: this.options.port || 3000,
      hostname: this.options.hostname || "localhost",
      routes: this.routesTable,
    });
    return this.server;
  }

  stop() {
    if (this.server) {
      this.server.stop();
    }
  }
}

/**
 * Type-safe route URI builder.
 */
export function uri(pattern, params = {}) {
  return native.buildRouteUri(pattern, params);
}

/**
 * Decorator factory for HTTP method attributes.
 */
function createRouteDecorator(method) {
  return function (path) {
    return function (target, propertyKey, descriptor) {
      const fn = descriptor ? descriptor.value : target;
      const routeInfo = { method: method.toUpperCase(), path, handler: fn };
      _registeredRoutes.push(routeInfo);
      native.registerRoute(method, path, fn);
      return descriptor;
    };
  };
}

export const get = createRouteDecorator("GET");
export const post = createRouteDecorator("POST");
export const put = createRouteDecorator("PUT");
export const del = createRouteDecorator("DELETE");
export const route = function (method, path) {
  return createRouteDecorator(method)(path);
};

export function catchCatcher(code) {
  return function (target, propertyKey, descriptor) {
    const fn = descriptor ? descriptor.value : target;
    _registeredCatchers.push({ code, handler: fn });
    return descriptor;
  };
}

export function routes() {
  return _registeredRoutes;
}

export function catchers() {
  return _registeredCatchers;
}

const Alloy = {
  main: native.main,
  serve: native.serve,
  file: native.file,
  response: native.response,
  Response: native.Response,
  Config: native.Config,
  App,
  uri,
  routes,
  catchers,
  get,
  post,
  put,
  route,
  catchCatcher,
};

export default Alloy;
