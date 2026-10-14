import { Alloy, responseJson, Config } from "../dist/index.js";

const config = Config.debugDefault();
config.port = 8080;
config.logLevel = "normal";

const server = Alloy.serve({
  port: config.port,
  routes: {
    "/config": () =>
      responseJson({
        profile: config.profile,
        port: config.port,
        address: config.address,
        logLevel: config.logLevel,
      }),
  },
});

console.log(`Configured server running at ${server.url}`);
