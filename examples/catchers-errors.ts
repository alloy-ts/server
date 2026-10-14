/**
 * Example: Custom Rocket Catchers & Error Handling
 *
 * Demonstrates:
 * 1. Creating custom `Catcher` instances for HTTP status codes (404, 500, default)
 * 2. Status code validation (400-599)
 * 3. Inspecting `isDefault` flag and modifying mount point via `mapBase`
 */

import { Catcher } from "../dist/index.js";

// 1. Specific HTTP error catchers
const notFoundCatcher = new Catcher(404, "/", "not_found_handler");
console.log(
  `404 Catcher Code: ${notFoundCatcher.code}, Base: ${notFoundCatcher.base}, Default: ${notFoundCatcher.isDefault}`,
);

const serverErrorCatcher = new Catcher(500, "/api", "internal_error_handler");
console.log(`500 Catcher Code: ${serverErrorCatcher.code}, Base: ${serverErrorCatcher.base}`);

// 2. Default fallback error catcher
const defaultCatcher = new Catcher(null, "/", "global_default_catcher");
console.log(`Default Catcher isDefault: ${defaultCatcher.isDefault}`);

// 3. Modifying base mount point via mapBase
notFoundCatcher.mapBase("/v1");
console.log(`Updated 404 Catcher Base: ${notFoundCatcher.base}`);

// 4. Invalid status code validation demonstration
try {
  new Catcher(200, "/");
} catch (e: any) {
  console.log("Validation caught expected error for 200 OK code:", e.message);
}

console.log("Catchers example executed successfully.");
