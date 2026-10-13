import { Alloy, Response } from "../src/main.ts";

interface Todo {
  id: string;
  title: string;
  completed: boolean;
}

const todos: Map<string, Todo> = new Map();

// Seed initial item
todos.set("1", { id: "1", title: "Implement Rocket NAPI Bindings", completed: true });

const server = Alloy.serve({
  port: 8080,
  routes: {
    // List all items or create new item
    "/api/todos": {
      GET: () => Response.json(Array.from(todos.values())),
      POST: async (req: any) => {
        try {
          const body = req.body ? JSON.parse(req.body.toString()) : {};
          if (!body.title) {
            return Response.json({ error: "Title is required" }, { status: 400 });
          }
          const id = String(todos.size + 1);
          const newTodo: Todo = { id, title: body.title, completed: false };
          todos.set(id, newTodo);
          return Response.json(newTodo, { status: 201 });
        } catch {
          return Response.json({ error: "Invalid JSON" }, { status: 400 });
        }
      },
    },

    // Get, update, or delete item by ID
    "/api/todos/:id": {
      GET: (req: any) => {
        const todo = todos.get(req.params.id);
        if (!todo) {
          return Response.json({ error: "Todo not found" }, { status: 404 });
        }
        return Response.json(todo);
      },
      PUT: async (req: any) => {
        const todo = todos.get(req.params.id);
        if (!todo) {
          return Response.json({ error: "Todo not found" }, { status: 404 });
        }
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        if (typeof body.completed === "boolean") {
          todo.completed = body.completed;
        }
        if (body.title) {
          todo.title = body.title;
        }
        return Response.json(todo);
      },
      DELETE: (req: any) => {
        if (!todos.has(req.params.id)) {
          return Response.json({ error: "Todo not found" }, { status: 404 });
        }
        todos.delete(req.params.id);
        return new Response(null, { status: 204 });
      },
    },
  },

  // Fallback 404 handler
  fetch(_req: any) {
    return Response.json({ error: "Route not found" }, { status: 404 });
  },
});

console.log(`REST API Server running at ${server.url}`);
