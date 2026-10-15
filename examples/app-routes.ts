import { serve, Response } from "../index.js";

const app = serve({
  routes: {
    "/": new Response("Hello World"),
    "/users/:id": (req) => {
      return new Response(`Hello ${req.params?.id}!`);
    },
  },
});

console.log(`Server running at ${app.url}`);
