import { Alloy, Response } from "../src/main.ts";

interface Post {
  id: string;
  title: string;
  content: string;
}

const postsDb: Map<string, Post> = new Map([
  ["1", { id: "1", title: "Welcome to Rocket", content: "Rocket web framework in TypeScript" }],
]);

const server = Alloy.serve({
  routes: {
    "/api/posts": {
      // List posts
      GET: () => Response.json(Array.from(postsDb.values())),

      // Create post
      POST: async (req: any) => {
        const payload = req.body ? JSON.parse(Buffer.from(req.body).toString()) : {};
        const id = String(postsDb.size + 1);
        const newPost: Post = {
          id,
          title: payload.title || "Untitled",
          content: payload.content || "",
        };
        postsDb.set(id, newPost);
        return Response.json(newPost, { status: 201 });
      },
    },

    // Get post by ID
    "/api/posts/:id": (req: any) => {
      const post = postsDb.get(req.params?.id);
      if (!post) {
        return Response.json({ error: "Post not found" }, { status: 404 });
      }
      return Response.json(post);
    },
  },

  fetch(req: any) {
    return Response.json({ error: `Path ${req.path} not found` }, { status: 404 });
  },

  port: 3000,
});

console.log(`REST API running at ${server.url}`);
