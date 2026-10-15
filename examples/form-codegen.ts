import { Codegen, buildUri } from "../dist/index.js";

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

const parsedRoute = Codegen.parseRoute("POST", "/submit/<form>?rank=1");

console.log("=== Parsed Route Metadata ===");
console.log(parsedRoute);

const uri = buildUri("/person/<name>?<age>", { name: "Alice", age: "25" }, "/v1");
console.log("=== Built Route URI ===");
console.log(uri);
