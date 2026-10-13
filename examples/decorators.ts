import { Alloy, Delete, Get, getRegisteredRoutesMeta, Post, Put, Response } from "../src/main.ts";

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

  @Put("/users/:id")
  updateUser(req: any) {
    return Response.json({ id: req.params.id, updated: true });
  }

  @Delete("/users/:id")
  deleteUser(req: any) {
    return Response.json({ id: req.params.id, deleted: true });
  }
}

// Instantiate controller to run property decorators
new UserController();

console.log("Registered routes metadata:");
console.dir(getRegisteredRoutesMeta());

const server = Alloy.serve({
  port: 8081,
  fetch() {
    return new Response("Decorator-based Rocket API Server");
  },
});

console.log(`Decorator example server running at ${server.url}`);
