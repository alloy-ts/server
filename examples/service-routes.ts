import { Service, response } from "@alloy-ts/service";
const service = new Service();

service.get("/", async () => response("Hello, world!"));
service.launch();

console.log("Service running at http://localhost:3000");

