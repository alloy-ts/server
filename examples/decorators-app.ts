import { Rocket, Route, Catcher, Codegen, buildUri } from "../dist/index.js";

/**
 * Demonstrates Rocket routes, catchers, lifecycle, and URI building.
 */
const route1 = new Route("GET", "/articles/<id>", "getArticle");
const route2 = new Route("POST", "/articles", "createArticle");

const catcher1 = new Catcher(404, "notFound");

const rocket = Rocket.build();
rocket.mount("/api/v1", [route1, route2]);
rocket.register("/api/v1", [catcher1]);

console.log("Mounted routes count:", rocket.routes().length);
console.log("Registered catchers count:", rocket.catchers().length);

rocket.ignite();
console.log("Rocket phase after ignite:", rocket.phase);

const articleUri = buildUri("/articles/<id>?<view>", { id: "101", view: "summary" }, "/v1");
console.log("Generated type-safe URI:", articleUri);

const meta = Codegen.parseRoute("GET", "/articles/<id>?<view>");
console.log("Parsed route metadata:", meta);
