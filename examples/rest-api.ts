import { responseJson, serve } from "../dist/index.js";

interface User {
  id: string;
  name: string;
  email: string;
}

const users = new Map<string, User>([
  ["1", { id: "1", name: "Alice", email: "alice@example.com" }],
  ["2", { id: "2", name: "Bob", email: "bob@example.com" }],
]);

const server = serve({
  port: 3000,
  hostname: "127.0.0.1",
  routes: {
    "/api/users": {
      // List all users
      GET: () => {
        return responseJson(Array.from(users.values()));
      },

      // Create a user
      POST: async (req) => {
        const bodyText = req.body ? new TextDecoder().decode(Uint8Array.from(req.body)) : "{}";
        const input = JSON.parse(bodyText) as Partial<User>;

        if (!input.name || !input.email) {
          return responseJson({ error: "Name and email are required" }, { status: 400 });
        }

        const id = String(users.size + 1);
        const newUser: User = { id, name: input.name, email: input.email };
        users.set(id, newUser);

        return responseJson(newUser, { status: 201 });
      },
    },

    // Get, update, or delete a user by ID
    "/api/users/:id": {
      GET: (req) => {
        const id = req.params["id"] ?? "";
        const user = users.get(id);
        if (!user) {
          return responseJson({ error: "User not found" }, { status: 404 });
        }
        return responseJson(user);
      },

      PUT: async (req) => {
        const id = req.params["id"] ?? "";
        const user = users.get(id);
        if (!user) {
          return responseJson({ error: "User not found" }, { status: 404 });
        }

        const bodyText = req.body ? new TextDecoder().decode(Uint8Array.from(req.body)) : "{}";
        const updates = JSON.parse(bodyText) as Partial<User>;

        const updated: User = {
          ...user,
          ...updates,
          id,
        };
        users.set(id, updated);

        return responseJson(updated);
      },

      DELETE: (req) => {
        const id = req.params["id"] ?? "";
        const deleted = users.delete(id);
        if (!deleted) {
          return responseJson({ error: "User not found" }, { status: 404 });
        }
        return responseJson({ success: true, deletedId: id });
      },
    },
  },

  fetch(req) {
    return responseJson({ error: "Route not found", path: req.path }, { status: 404 });
  },
});

console.log(`User REST API running at ${server.url}`);
