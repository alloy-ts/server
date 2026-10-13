import { Alloy, Response } from "../dist/index.js";

/**
 * Example demonstrating asynchronous I/O, simulated database queries,
 * external API fetching, and error handling.
 */

// Simulated async database query
async function fetchUserFromDb(userId: string) {
  // Simulate DB network latency
  await new Promise((resolve) => setTimeout(resolve, 50));

  if (userId === "42") {
    return { id: "42", name: "Douglas Adams", role: "Author" };
  }
  return null;
}

const server = Alloy.serve({
  port: 3007,
  routes: {
    "/users/:id": async (req) => {
      try {
        const userId = req.params["id"];
        const user = await fetchUserFromDb(userId);

        if (!user) {
          return Response.json({ error: `User with ID '${userId}' not found` }, { status: 404 });
        }

        return Response.json({ success: true, data: user });
      } catch (err: any) {
        return Response.json(
          { error: "Internal Server Error", message: err?.message },
          { status: 500 },
        );
      }
    },

    // Async data processing handler
    "/api/process": {
      POST: async (req) => {
        const bodyStr = req.body ? new TextDecoder().decode(new Uint8Array(req.body)) : "{}";
        const body = JSON.parse(bodyStr);

        // Perform async calculations/processing
        await new Promise((resolve) => setTimeout(resolve, 30));

        return Response.json({
          processed: true,
          received: body,
          timestamp: new Date().toISOString(),
        });
      },
    },
  },
});

console.log(`Async Operations Example Server running at ${server.url}`);
