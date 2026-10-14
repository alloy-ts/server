import { Alloy, responseJson } from "../dist/index.js";

const posts = [
  { id: "1", title: "First Post", content: "Hello AlloyScript!" },
  { id: "2", title: "Rocket NAPI", content: "High performance web server" },
];

const server = Alloy.serve({
  port: 3000,
  routes: {
    "/api/posts": {
      GET: () => responseJson({ posts }),
      POST: async (req: any) => {
        const body = req.json();
        const newPost = { id: String(posts.length + 1), ...body };
        posts.push(newPost);
        return responseJson({ created: true, post: newPost }, { status: 201 });
      },
    },
    "/api/posts/:id": (req: any) => {
      const post = posts.find((p) => p.id === req.params.id);
      if (!post) {
        return responseJson({ error: "Post not found" }, { status: 404 });
      }
      return responseJson({ post });
    },
  },
});

console.log(`JSON API Server running at ${server.url}`);
