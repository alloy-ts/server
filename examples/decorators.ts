import { registerRoute, Response, serve } from "../dist/index.js";

function Get(path: string) {
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor,
  ): PropertyDescriptor {
    registerRoute("GET", path, descriptor.value);
    return descriptor;
  };
}

function Post(path: string) {
  return function (
    _target: any,
    _propertyKey: string,
    descriptor: PropertyDescriptor,
  ): PropertyDescriptor {
    registerRoute("POST", path, descriptor.value);
    return descriptor;
  };
}

class UserController {
  @Get("/users")
  listUsers() {
    return Response.json([
      { id: "1", name: "Alice" },
      { id: "2", name: "Bob" },
    ]);
  }

  @Get("/users/:id")
  getUser(req: any) {
    return Response.json({ id: req.params.id, name: "Alice" });
  }

  @Post("/users")
  createUser(req: any) {
    const data = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
    return Response.json({ id: "3", ...data }, { status: 201 });
  }
}

new UserController();

const server = serve(null, null, { port: 8081 });

console.log(`Decorator example server running at ${server.url}`);
