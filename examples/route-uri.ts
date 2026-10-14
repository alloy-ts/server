/**
 * Example: Inspecting Route URIs and Rocket Ranking Table with RouteUri
 *
 * Demonstrates:
 * 1. Instantiating `RouteUri` with base mount points and origins
 * 2. Extracting path and query parts
 * 3. Computing default route ranks according to Rocket's default ranking table
 */

import { RouteUri } from "../dist/index.js";

// 1. Static route URI with base mount point
const uri1 = new RouteUri("/api/v1", "/status?verbose=true");
console.log("Base:", uri1.base);
console.log("Unmounted Origin:", uri1.unmountedOrigin);
console.log("Canonical Origin:", uri1.origin);
console.log("Path:", uri1.path());
console.log("Query:", uri1.query());
console.log("Default Rank (Static):", uri1.defaultRank()); // -12

// 2. Dynamic wildcard route URI
const uri2 = new RouteUri("/", "/users/<b>/<c>");
console.log("\nDynamic Route Path:", uri2.path());
console.log("Default Rank (Wildcard):", uri2.defaultRank()); // -1

// 3. Partial static / dynamic route URI
const uri3 = new RouteUri("/base", "/posts/<id>?<ref>");
console.log("\nPartial Route Path:", uri3.path());
console.log("Query:", uri3.query());
console.log("Default Rank (Partial):", uri3.defaultRank());

console.log("\nRouteUri example executed successfully.");
