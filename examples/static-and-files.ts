import { Alloy, Response } from "../dist/index.js";

/**
 * Example demonstrating static file handling, lazy file loading,
 * and serving custom HTML/CSS content.
 */

const server = Alloy.serve({
  port: 3006,
  routes: {
    // Serve HTML string directly
    "/": () => {
      const htmlContent = `
        <!DOCTYPE html>
        <html lang="en">
        <head>
          <meta charset="UTF-8" />
          <title>AlloyScript Rocket Server</title>
          <style>
            body { font-family: system-ui, sans-serif; background: #0f172a; color: #f8fafc; padding: 3rem; }
            h1 { color: #38bdf8; }
            code { background: #1e293b; padding: 0.2rem 0.4rem; border-radius: 4px; }
          </style>
        </head>
        <body>
          <h1>Welcome to AlloyScript Web Server!</h1>
          <p>Powered by Rust 🚀 Rocket NAPI bindings for Node.js.</p>
          <p>Check out <code>/package.json</code> or <code>/api/health</code>!</p>
        </body>
        </html>
      `;
      return new Response(htmlContent, {
        headers: { "content-type": "text/html; charset=utf-8" },
      });
    },

    // Lazy static file loading using Alloy.file
    "/package.json": Alloy.file("./package.json"),

    // Health check endpoint
    "/api/health": Response.json({ status: "ok", timestamp: Date.now() }),
  },

  fetch() {
    return new Response("Page or File Not Found", { status: 404 });
  },
});

console.log(`Static & Files Example Server running at ${server.url}`);
