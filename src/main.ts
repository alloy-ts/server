export * from "../dist/index.js";

import { serve, file, registerRoute, launch, Response } from "../dist/index.js";

export const Alloy = {
  serve,
  file,
};

export function response(
  body: string | object,
  status = 200,
  headers?: Record<string, string>,
): Response {
  if (typeof body === "object") {
    return Response.json(body, {
      status,
      headers: headers
        ? Object.entries(headers).map(([name, value]) => ({ name, value }))
        : undefined,
    });
  }
  return new Response(body, {
    status,
    headers: headers
      ? Object.entries(headers).map(([name, value]) => ({ name, value }))
      : undefined,
  });
}

export class App {
  private routes: Array<{ method: string; path: string; handler: any }> = [];

  get(path: string, handler: any): this {
    this.routes.push({ method: "GET", path, handler });
    return this;
  }

  post(path: string, handler: any): this {
    this.routes.push({ method: "POST", path, handler });
    return this;
  }

  put(path: string, handler: any): this {
    this.routes.push({ method: "PUT", path, handler });
    return this;
  }

  delete(path: string, handler: any): this {
    this.routes.push({ method: "DELETE", path, handler });
    return this;
  }

  patch(path: string, handler: any): this {
    this.routes.push({ method: "PATCH", path, handler });
    return this;
  }

  mount(basePath: string, routes: Array<{ method: string; path: string; handler: any }>): this {
    for (const route of routes) {
      const fullPath = (basePath.replace(/\/$/, "") + "/" + route.path.replace(/^\//, "")).replace(
        /\/+/g,
        "/",
      );
      this.routes.push({ ...route, path: fullPath });
    }
    return this;
  }

  async launch(): Promise<void> {
    for (const route of this.routes) {
      registerRoute(route.method, route.path, route.handler);
    }
    await launch();
  }
}

export function main(): string {
  return "Hello, world!";
}

export default Alloy;
