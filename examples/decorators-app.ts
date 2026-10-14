import { Get, Post, Route, Catch, routes, catchers, uri, responseJson } from "../src/main.ts";

/**
 * Demonstrates Rocket route decorators, catchers, and URI builder.
 */
class ArticleController {
  @Get("/articles/<id>")
  getArticle() {
    return responseJson({ id: "article-1", title: "Rocket Web Framework" });
  }

  @Post("/articles")
  createArticle() {
    return responseJson({ created: true });
  }

  @Route("PUT", "/articles/<id>")
  updateArticle() {
    return responseJson({ updated: true });
  }

  @Catch(404)
  notFound() {
    return responseJson({ error: "Article Not Found" }, { status: 404 });
  }
}

const controller = new ArticleController();
const registeredRoutes = routes(
  controller.getArticle,
  controller.createArticle,
  controller.updateArticle,
);
const registeredCatchers = catchers(controller.notFound);

console.log("Registered routes count:", registeredRoutes.length);
console.log("Registered catchers count:", registeredCatchers.length);

const articleUri = uri("/articles/<id>?<view>", { id: 101, view: "summary" }, "/v1");
console.log("Generated type-safe URI:", articleUri);
