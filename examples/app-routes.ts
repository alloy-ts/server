import { App, response } from "@alloy-ts/server";
const app = new App();

app.get("/", async () => response("Hello, world!"));
app.launch();

console.log("Server running at http://localhost:3000");
