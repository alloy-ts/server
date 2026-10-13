import { App, responseJson } from "../src/main.ts";

interface Item {
  id: string;
  name: string;
  price: number;
}

const db = new Map<string, Item>([
  ["1", { id: "1", name: "Laptop", price: 999.99 }],
  ["2", { id: "2", name: "Mouse", price: 29.99 }],
]);

const app = new App({ port: 3000 });

// GET /items - List items
app.get("/items", async () => {
  return responseJson(Array.from(db.values()));
});

// GET /items/:id - Get item by ID
app.get("/items/:id", async (req) => {
  const item = db.get(req.params?.id);
  if (!item) {
    return responseJson({ error: "Item not found" }, { status: 404 });
  }
  return responseJson(item);
});

// POST /items - Create item
app.post("/items", async (req) => {
  let body: any = {};
  if (req.body) {
    try {
      body = JSON.parse(Buffer.from(req.body).toString("utf-8"));
    } catch {
      return responseJson({ error: "Invalid JSON body" }, { status: 400 });
    }
  }
  const id = String(db.size + 1);
  const newItem: Item = { id, name: body.name ?? "New Item", price: body.price ?? 0 };
  db.set(id, newItem);
  return responseJson(newItem, { status: 201 });
});

// DELETE /items/:id - Delete item
app.delete("/items/:id", async (req) => {
  const deleted = db.delete(req.params?.id);
  if (!deleted) {
    return responseJson({ error: "Item not found" }, { status: 404 });
  }
  return responseJson({ message: "Item deleted successfully" });
});

app.launch();
console.log("REST API Server running at http://localhost:3000");
