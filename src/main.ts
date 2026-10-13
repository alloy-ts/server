export * from "../dist/index.js";
import * as native from "../dist/index.js";

export const Alloy = {
  serve: native.serve,
  file: native.file,
  response: native.response,
  responseJson: native.responseJson,
  responseRedirect: native.responseRedirect,
};

export function main(): string {
  return "Hello, world!";
}
