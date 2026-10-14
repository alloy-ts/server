# `@alloy-ts/server` Implementation Status

This document provides a summary of the implementation status for the Rocket web framework NAPI bindings and TypeScript code generation utilities in `@alloy-ts/server` (`alloy-server-native`).

---

## Architecture Summary

`@alloy-ts/server` is a hybrid Node.js / Rust package using [NAPI-RS](https://napi.rs/) to connect Node.js / TypeScript applications directly to the [Rocket](https://rocket.rs/) web framework.

- **Crate Name**: `alloy-server-native`
- **NAPI Binary**: `alloy-server.<platform-arch>.node`
- **TypeScript Entry Point**: `./dist/index.js` (with type definitions `./dist/index.d.ts`)

---

## Implemented NAPI Binding Modules

### 1. HTTP Server Runtime (`src/server.rs`)

- **`serve(routes, fetch, options)`**: Bun/Alloy-style server initializer backed by Rocket's multi-threaded Tokio runtime.
  - Supports static responses, dynamic path parameter extraction (`/users/:id`), per-HTTP method maps (`{ GET, POST }`), wildcards (`/api/*`), file loading (`file()`), and redirect responses (`responseRedirect()`).
  - Accepts fallback `fetch()` handler and custom server options (`port`, `hostname`).
  - Returns `Server` handle with `url`, `port`, and `.stop()` graceful shutdown.
- **`Response` & Constructors**:
  - `response(body?, init?)`: Plain response object constructor.
  - `responseJson(data, init?)`: Helper returning JSON response (`Content-Type: application/json`).
  - `responseRedirect(url, status?)`: Helper returning redirect response.
  - `file(path)`: Static file marker.
- **`registerRoute(method, path, handler)`**: Low-level route registration function.

### 2. Rocket Lifecycle Management (`src/main.rs`)

- **`Rocket` Class**: Mirror of Rocket's server lifecycle:
  - `new Rocket(config?)`: Instantiate server with `RocketConfig`.
  - `mount(base, method, path, handler)`: Programmatically mount individual route handlers.
  - `register(base, code, handler)`: Programmatically register error catchers.
  - `launch()`: Ignite and launch Rocket server on the Tokio runtime.
  - `shutdown()`: Initiate graceful server shutdown.
- **Top-Level Helpers**: `launch(config?)` and `rocketMain(config?)`.

### 3. Rocket Configuration (`src/config.rs`)

- **`Config` Class**: Faithful NAPI mirror of `rocket::config::Config`:
  - Static factory methods: `Config.debugDefault()`, `Config.releaseDefault()`, `Config.defaultConfig()`, `Config.figment()`, `Config.tryFrom(init)`, `Config.from(init)`.
  - All standard configuration parameter getters/setters: `profile`, `address`, `port`, `workers`, `maxBlocking`, `keepAlive`, `ident`, `ipHeader`, `limits`, `tempDir`, `shutdown`, `logLevel`, `cliColors`.
  - Introspection method: `Config.parameters()`.
- **Composite Value Objects**:
  - `Limits`: Map of data-type byte limits.
  - `Shutdown`: Graceful shutdown options (`ctrlc`, `signals`, `grace`, `mercy`, `force`).
  - `ConfigInit`: Partial configuration override initializer.
- **Associated Constants**: `DEBUG_PROFILE`, `RELEASE_PROFILE`, `DEFAULT_PROFILE`, `ADDRESS`, `PORT`, `WORKERS`, `MAX_BLOCKING`, `KEEP_ALIVE`, `IDENT`, `IP_HEADER`, `LIMITS`, `TLS`, `SECRET_KEY`, `TEMP_DIR`, `LOG_LEVEL`, `SHUTDOWN`, `CLI_COLORS`.

### 4. Code Generation (`src/codegen.rs`)

- **Route Pattern Utilities**:
  - `parseRoute(input)`: Parse single pattern string into resolved `RouteDef` with parameter metadata, handler name derivation, and parameter sanitization (e.g. `type` -> `type_`).
  - `parseRoutes(inputs)`: Parse multiple route patterns and deduplicate generated handler names (e.g. `getAById` and `getAById_2`).
  - `toRocketPath(pattern)`: Translate Bun/Alloy route syntax into Rocket syntax (`/users/:id/*` -> `/users/<id>/<_..>`).
  - `httpMethods()`: Return list of HTTP methods supported by Rocket route attributes.
- **Code Emitters**:
  - `generateTypes(defs, options)`: Emit TypeScript route map interfaces (`RouteTypes`, `ParamsOf`, `QueryOf`, `BodyOf`, `ResponseOf`, `RouteContext`, `RouteHandler`).
  - `generateClient(defs, options)`: Emit typed `fetch` client class and `uri()` route builder helper.
  - `generateModule(defs, options)`: Emit full TypeScript module containing `ROUTES`, types, `uri()` helper, and client class.

---

## Test Coverage

### TypeScript Unit Tests

- **`src/codegen.test.ts`** (13 tests):
  - `httpMethods`: Verifies supported Rocket HTTP methods.
  - `toRocketPath`: Verifies Bun/Alloy to Rocket path translation.
  - `parseRoute`: Verifies path/query extraction, trailing/optional flags, reserved JS keyword sanitization, and invalid method error handling.
  - `parseRoutes`: Verifies handler name deduplication.
  - `generateTypes`: Verifies `.d.ts` route map interface emission and `serverImport` options.
  - `generateClient`: Verifies fetch client class emission and `typesImport` options.
  - `generateModule`: Verifies full module code generation and banner options.
- **`src/config.test.ts`** (7 tests):
  - Constants: Verifies export of configuration parameter names and profiles.
  - Factories: Tests `debugDefault()`, `releaseDefault()`, `tryFrom()`, and `parameters()`.
  - Mutations: Tests getter/setter updates for scalars and composite objects (`shutdown`), as well as validation rules for address and log levels.

### Rust Unit Tests

- **`src/codegen.rs`** (8 unit tests):
  - `parses_bun_style_path`
  - `parses_rocket_style_path_and_query`
  - `root_and_unknown_methods`
  - `lists_http_methods`
  - `sanitizes_reserved_and_invalid_params`
  - `dedupes_route_names`
  - `translates_paths`
  - `emits_typescript`

---

## Code Quality & Verification

- **Linting & Type Checking**: `npm run check` (`vp check`) passes with **0 errors and 0 warnings**.
- **Formatting**: `npm run fmt` (`vp fmt`) passes cleanly across all codebase files.
- **Native Build**: `npm run build:napi` builds native binary and generates updated `./dist/index.js` and `./dist/index.d.ts`.
