import { Codegen, parseRoutePattern } from "../dist/index.js";

// Parse Rocket route parameters
const pattern = "/users/<id>/posts/<postId>?<sort>&<limit>";
const parsed = parseRoutePattern(pattern);

console.log("Path parameters:", parsed.path_params);
console.log("Query parameters:", parsed.query_params);

// Register route metadata and generate TypeScript code
const codegen = new Codegen();

codegen.addRoute({
  method: "GET",
  path: "/api/users/<id>",
  name: "get_user_by_id",
  pathParams: ["id"],
  queryParams: [],
});

codegen.addRoute({
  method: "POST",
  path: "/api/users",
  name: "create_user",
  dataParam: "user_data",
  pathParams: [],
  queryParams: [],
});

const output = codegen.generateTs();
console.log("\nGenerated TypeScript Code:");
console.log(output.tsCode);
