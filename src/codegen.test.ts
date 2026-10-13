import { expect, test } from "vite-plus/test";
import { uri, Get, Post, Put, Delete, Patch, Head, Options, Route, Catch } from "./main.ts";

test("uri replaces path parameters correctly", () => {
  const result1 = uri("/person/<name>", { name: "Mike Smith" });
  expect(result1).toBe("/person/Mike%20Smith");

  const result2 = uri("/users/:id/posts/<postId>", { id: 42, postId: "hello-world" });
  expect(result2).toBe("/users/42/posts/hello-world");
});

test("uri returns original pattern if no params provided", () => {
  const pattern = "/api/status";
  expect(uri(pattern)).toBe(pattern);
});

test("route decorators register functions cleanly", () => {
  class Controller {
    @Get("/items")
    getItems() {
      return "items";
    }

    @Post("/items")
    createItem() {
      return "created";
    }

    @Put("/items/:id")
    updateItem() {
      return "updated";
    }

    @Delete("/items/:id")
    deleteItem() {
      return "deleted";
    }

    @Patch("/items/:id")
    patchItem() {
      return "patched";
    }

    @Head("/items")
    headItems() {
      return "";
    }

    @Options("/items")
    optionsItems() {
      return "";
    }

    @Route("GET", "/custom")
    customRoute() {
      return "custom";
    }

    @Catch(404)
    notFound() {
      return "not found";
    }
  }

  const instance = new Controller();
  expect(typeof instance.getItems).toBe("function");
});
