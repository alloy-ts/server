/**
 * Example: Database / In-Memory REST API
 *
 * Demonstrates:
 * 1. CRUD operations for a resource (`/api/todos`)
 * 2. In-memory state management
 * 3. Handling GET, POST, DELETE requests
 * 4. Path parameter parsing (`req.params.id`)
 */

import { Alloy, Response } from "../dist/main.mjs";

interface Todo {
  id: string;
  title: string;
  completed: boolean;
}

const todos: Map<string, Todo> = new Map([
  ["1", { id: "1", title: "Learn Rocket NAPI", completed: true }],
  ["2", { id: "2", title: "Build TypeScript Server", completed: false }],
]);

const server = Alloy.serve({
  port: 8080,
  routes: {
    "/api/todos": {
      // List all todos
      GET: () => Response.json(Array.from(todos.values())),

      // Create a todo
      POST: async (req: any) => {
        const id = String(todos.size + 1);
        const newTodo: Todo = { id, title: "New Task", completed: false };
        todos.set(id, newTodo);
        return Response.json(newTodo, { status: 201 });
      },
    },

    // Get or Delete todo by ID
    "/api/todos/:id": {
      GET: (req: any) => {
        const item = todos.get(req.params.id);
        if (!item) {
          return new Response("Todo Not Found", { status: 404 });
        }
        return Response.json(item);
      },
      DELETE: (req: any) => {
        const existed = todos.delete(req.params.id);
        if (!existed) {
          return new Response("Todo Not Found", { status: 404 });
        }
        return Response.json({ deleted: true });
      },
    },
  },

  fetch() {
    return new Response("404 Not Found", { status: 404 });
  },
});

console.log(`REST API Server running at ${server.url}`);

setTimeout(() => {
  server.stop();
}, 2000);
