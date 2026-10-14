import { CustomHandler } from "../dist/index.js";

// Custom request handler implementing Rocket's request Handler interface
const handler = new CustomHandler(async (req: any) => {
  console.log(`Handling ${req.method} request to ${req.path}`);

  return {
    status: "success",
    response: {
      status: 200,
      headers: { "Content-Type": "text/plain", "X-Custom-Header": "RocketNAPI" },
      body: Array.from(Buffer.from(`Handled request to ${req.path}`)),
    },
    errorCode: null,
  };
});

console.log("CustomHandler instantiated successfully:", handler);
