import Alloy, { Response } from "../index.js";

interface Post {
  id: string;
  title: string;
  content: string;
  createdAt: string;
}

const posts: Map<string, Post> = new Map();

posts.set("1", {
  id: "1",
  title: "Hello Rocket",
  content: "Rocket web framework in Node.js via NAPI",
  createdAt: new Date().toISOString(),
});

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/api/posts": {
      GET: () => Response.json(Array.from(posts.values())),
      POST: async (req: any) => {
        const body = req.body ? JSON.parse(req.body.toString()) : {};
        const id = String(posts.size + 1);
        const newPost: Post = {
          id,
          title: body.title || "Untitled",
          content: body.content || "",
          createdAt: new Date().toISOString(),
        };
        posts.set(id, newPost);
        return Response.json(newPost, { status: 201 });
      },
    },

    "/api/posts/:id": (req: any) => {
      const id = req.params.id;
      const post = posts.get(id);
      if (!post) {
        return new Response("Not Found", { status: 404 });
      }
      return Response.json(post);
    },
  },

  fetch(_req: any) {
    return Response.json({ error: "Route not found" }, { status: 404 });
  },
});

console.log(`REST API server running at ${server.url}`);
