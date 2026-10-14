import { get, post, route, catchCatcher, routes, catchers, launch, Response } from "../index.js";

class UserController {
  @get("/users")
  listUsers() {
    return Response.json([
      { id: "1", name: "Alice" },
      { id: "2", name: "Bob" },
    ]);
  }

  @post("/users")
  createUser() {
    return Response.json({ created: true }, { status: 201 });
  }

  @route("PUT", "/users/:id")
  updateUser() {
    return Response.json({ updated: true });
  }

  @catchCatcher(404)
  notFound() {
    return new Response("User not found", { status: 404 });
  }
}

new UserController();

console.log("Registered routes:");
for (const r of routes()) {
  console.log(`  ${r.method} ${r.path}`);
}

console.log("Registered catchers:");
for (const c of catchers()) {
  console.log(`  Status ${c.code ?? "default"}`);
}

launch();
