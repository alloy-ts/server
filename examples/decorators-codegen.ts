import { registerRoute, uri, launch, response } from "../dist/index.js";

// Register routes using Rocket NAPI bindings
registerRoute("GET", "/articles", async () => response("List of articles"));
registerRoute("GET", "/articles/:id", async (req: any) =>
  response(`Article ID: ${req.params?.id}`),
);
registerRoute("POST", "/articles", async () => response("Article created"));
registerRoute("DELETE", "/articles/:id", async (req: any) =>
  response(`Deleted article ${req.params?.id}`),
);

// Construct type-safe URIs
const articleUri = uri("/articles/:id", { id: 101 });
console.log(`Generated URI: ${articleUri}`);

// Launch Rocket server
void launch();
console.log("Server running at http://localhost:3000");
