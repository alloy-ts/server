import { Route, RouteUri, uri } from "../index.js";

// 1. Generate parameterized URIs using `uri` helper
const userProfileUri = uri("/users/:id/profile", { id: "42" });
console.log("Generated User Profile URI:", userProfileUri);

const postCommentUri = uri("/posts/<post_id>/comments/<comment_id>", {
  post_id: "101",
  comment_id: "5",
});
console.log("Generated Post Comment URI:", postCommentUri);

// 2. Inspect route metadata with `Route`
const route = new Route("GET", "/users/<id>", "get_user");
console.log("\nRoute Method:", route.method);
console.log("Route URI:", route.uri);
console.log("Route Name:", route.name);
console.log("Computed Route Rank:", route.rank);

// 3. Work with mounted route URIs using `RouteUri`
const routeUri = new RouteUri("/api/v1", "/users/<id>?active=true");
console.log("\nBase Path:", routeUri.base);
console.log("Unmounted Origin:", routeUri.unmountedOrigin);
console.log("Mounted Origin:", routeUri.origin);
console.log("Path Portion:", routeUri.path());
console.log("Query Portion:", routeUri.query());
