import { uri } from "../src/main.ts";

// Rocket uri! macro equivalent for TypeScript
console.log("=== Rocket URI Codegen Builder Examples ===");

// 1. Basic path parameter interpolation
const userUri = uri("/users/:id", { id: "42" });
console.log("User URI:", userUri); // "/users/42"

// 2. Multiple path parameters
const postCommentUri = uri("/posts/:postId/comments/:commentId", {
  postId: "101",
  commentId: "5",
});
console.log("Post Comment URI:", postCommentUri); // "/posts/101/comments/5"

// 3. Path parameters + query parameters
const searchUri = uri(
  "/search/:category",
  { category: "books" },
  { q: "rocket web framework", sort: "desc", page: "1" },
);
console.log("Search URI:", searchUri); // "/search/books?q=rocket%20web%20framework&sort=desc&page=1"

// 4. URL percent encoding special characters
const encodedUri = uri("/files/:filename", { filename: "hello world & rocket.png" });
console.log("Encoded URI:", encodedUri); // "/files/hello%20world%20%26%20rocket.png"
