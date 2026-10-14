import { expect, test } from "vite-plus/test";
import {
  type RequestInput,
  type Server,
  file,
  registerRoute,
  response,
  responseJson,
  responseRedirect,
  serve,
} from "../dist/index.js";

test("serve() HTTP server with route table and fetch fallback", async () => {
  const server: Server = serve(
    {
      "/status": response("OK"),
      "/json": async () => responseJson({ name: "Rocket" }),
      "/redirect": responseRedirect("/target", 302),
      "/users/:id": async (req: RequestInput) => response(`User ${req.params?.["id"]}`),
      "/api/posts": {
        GET: async () => response("Get Posts"),
        POST: async (req: RequestInput) => {
          let body = {};
          if (req.body) {
            body = JSON.parse(Buffer.from(req.body).toString("utf-8"));
          }
          return responseJson({ created: true, ...body });
        },
      },
      "/api/*": responseJson({ error: "API Not Found" }, { status: 404 }),
      "/file": file("./test.txt"),
    },
    async () => response("Fallback 404", { status: 404 }),
    { port: 8084 },
  );

  expect(server.port).toBe(8084);
  expect(server.url).toContain("http://");

  // 1. Static status route
  const resStatus = await fetch("http://127.0.0.1:8084/status");
  expect(resStatus.status).toBe(200);
  expect(await resStatus.text()).toBe("OK");

  // 2. JSON route
  const resJson = await fetch("http://127.0.0.1:8084/json");
  expect(resJson.status).toBe(200);
  expect(resJson.headers.get("content-type")).toContain("application/json");
  const jsonData = (await resJson.json()) as any;
  expect(jsonData.name).toBe("Rocket");

  // 3. Redirect route
  const resRedir = await fetch("http://127.0.0.1:8084/redirect", {
    redirect: "manual",
  });
  expect(resRedir.status).toBe(302);
  expect(resRedir.headers.get("location")).toBe("/target");

  // 4. Dynamic path parameter
  const resUser = await fetch("http://127.0.0.1:8084/users/99");
  expect(resUser.status).toBe(200);
  expect(await resUser.text()).toBe("User 99");

  // 5. Per-method GET
  const resGetPosts = await fetch("http://127.0.0.1:8084/api/posts");
  expect(resGetPosts.status).toBe(200);
  expect(await resGetPosts.text()).toBe("Get Posts");

  // 6. Per-method POST with JSON body
  const resPostPosts = await fetch("http://127.0.0.1:8084/api/posts", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ title: "Hello World" }),
  });
  expect(resPostPosts.status).toBe(200);
  const postData = (await resPostPosts.json()) as any;
  expect(postData.created).toBe(true);
  expect(postData.title).toBe("Hello World");

  // 7. Wildcard route
  const resWildcard = await fetch("http://127.0.0.1:8084/api/unknown/path");
  expect(resWildcard.status).toBe(404);

  server.stop();
});

test("registerRoute low-level API", () => {
  expect(typeof registerRoute).toBe("function");
});
