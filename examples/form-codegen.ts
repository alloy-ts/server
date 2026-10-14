import { Codegen } from "../src/main.ts";

/**
 * Demonstrates Rocket FromForm code generation and Form schema generation.
 */
const fields = [
  { name: "username", mappedName: "user_name", validateRule: "len(3..)" },
  { name: "email", validateRule: "email" },
  { name: "age", defaultValue: "18" },
];

const generatedSchema = Codegen.generateFormSchema(fields);

console.log("=== Generated Form Validation Schema ===");
console.log(generatedSchema);

const parsedRoute = Codegen.parseRoute(
  "POST",
  '"/submit", data = "<form>", rank = 1, format = "json"',
);

console.log("=== Parsed Route Metadata ===");
console.log(parsedRoute);
