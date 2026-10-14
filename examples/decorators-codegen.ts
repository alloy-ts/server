import { Get, Post, Delete, uri, launch } from "../src/main.ts";

class ArticleController {
  @Get("/articles")
  listArticles() {
    return "List of articles";
  }

  @Get("/articles/:id")
  getArticle(req: any) {
    return `Article ID: ${req.params?.id}`;
  }

  @Post("/articles")
  createArticle() {
    return "Article created";
  }

  @Delete("/articles/:id")
  deleteArticle(req: any) {
    return `Deleted article ${req.params?.id}`;
  }
}

// Instantiate controller to register route decorators
new ArticleController();

// Construct type-safe URIs
const articleUri = uri("/articles/:id", { id: 101 });
console.log(`Generated URI: ${articleUri}`);

// Launch Rocket server
void launch();
console.log("Decorators server running at http://localhost:3000");
