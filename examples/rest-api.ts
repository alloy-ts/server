import { Alloy, Response } from "../dist/index.js";

interface Todo {
  id: string;
  title: string;
  completed: boolean;
}

const todos: Map<string, Todo> = new Map([
  ["1", { id: "1", title: "Build Rocket NAPI bindings", completed: true }],
  ["2", { id: "2", title: "Support Bun.serve API in Alloy", completed: true }],
]);

const server = Alloy.serve({
  port: 3001,
  hostname: "localhost",
  routes: {
    // Health check
    "/api/health": new Response("OK"),

    // List and Create Todos
    "/api/todos": {
      GET: () => Response.json(Array.from(todos.values())),
      POST: async (req: any) => {
        const bodyStr = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        const payload = JSON.parse(bodyStr);
        if (!payload.title) {
          return Response.json({ error: "Title is required" }, { status: 400 });
        }
        const id = String(todos.size + 1);
        const todo: Todo = { id, title: payload.title, completed: false };
        todos.set(id, todo);
        return Response.json(todo, { status: 201 });
      },
    },

    // Get, Update, and Delete Todo by ID
    "/api/todos/:id": {
      GET: (req: any) => {
        const todo = todos.get(req.params.id);
        if (!todo) {
          return Response.json({ error: "Todo not found" }, { status: 404 });
        }
        return Response.json(todo);
      },
      DELETE: (req: any) => {
        const existed = todos.delete(req.params.id);
        if (!existed) {
          return Response.json({ error: "Todo not found" }, { status: 404 });
        }
        return Response.json({ deleted: true });
      },
    },

    // Wildcard fallback for /api/
    "/api/*": Response.json({ error: "API endpoint not found" }, { status: 404 }),
  },

  fetch(_req: any) {
    return new Response("Not Found", { status: 404 });
  },
});

console.log(`Todo REST API server running at ${server.url}`);
