import {
  generateRoutesManifest,
  buildUri,
  type RouteMetadata,
  type CatcherMetadata,
} from "../dist/index.js";

// 1. Declare route metadata matching Rocket procedural macros
const routes: RouteMetadata[] = [
  {
    name: "index",
    method: "GET",
    path: "/",
    rank: 0,
    format: "text/html",
  },
  {
    name: "get_user",
    method: "GET",
    path: "/users/<id>",
    rank: 0,
  },
  {
    name: "create_user",
    method: "POST",
    path: "/users",
    rank: 0,
    format: "application/json",
  },
];

// 2. Declare catcher metadata
const catchers: CatcherMetadata[] = [
  { name: "not_found", code: 404 },
  { name: "internal_error", code: 500 },
];

// 3. Generate routes manifest TypeScript / JSON export
const manifest = generateRoutesManifest(routes, catchers);
console.log("Generated Routes Manifest:");
console.log(manifest);

// 4. Build type-safe URIs at runtime
const userUri = buildUri("/users/<id>", { id: "42" });
console.log(`\nBuilt Route URI: ${userUri}`);

const searchUri = buildUri("/search/<query>?<page>", {
  query: "rocket & typescript",
  page: "1",
});
console.log(`Built Search URI: ${searchUri}`);
