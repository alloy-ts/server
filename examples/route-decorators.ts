import { get, post, del, serve, Response } from "../src/main.ts";

class UserController {
  @get("/users/:id")
  getUser(req: any) {
    return new Response(`User ID: ${req.params?.id}`);
  }

  @post("/users")
  async createUser(req: any) {
    const body = req.body ? JSON.parse(Buffer.from(req.body).toString("utf-8")) : {};
    return Response.json({ status: "created", user: body });
  }

  @del("/users/:id")
  deleteUser(req: any) {
    return Response.json({ deleted: req.params?.id });
  }
}

// Instantiate controller to run route registration decorators
new UserController();

// Launch server with registered decorator routes
const server = serve({
  port: 3000,
});

console.log(`Decorator routes server running at ${server.url}`);

setTimeout(() => {
  server.stop();
}, 1000);
