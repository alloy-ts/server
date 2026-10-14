#![recursion_limit = "256"]
#![doc(html_root_url = "https://api.rocket.rs/v0.5")]
#![doc(html_favicon_url = "https://rocket.rs/images/favicon.ico")]
#![doc(html_logo_url = "https://rocket.rs/images/logo-boxed.png")]
#![cfg_attr(nightly, feature(doc_cfg))]
#![cfg_attr(nightly, feature(decl_macro))]
#![warn(rust_2018_idioms)]
#![allow(missing_docs)]

//! # Rocket - Core API Documentation
//!
//! Hello, and welcome to the core Rocket API documentation!
//!
//! This API documentation is highly technical and is purely a reference.
//! There's an [overview] of Rocket on the main site as well as a [full,
//! detailed guide]. If you'd like pointers on getting started, see the
//! [quickstart] or [getting started] chapters of the guide.
//!
//! [overview]: https://rocket.rs/v0.5/overview
//! [full, detailed guide]: https://rocket.rs/v0.5/guide
//! [quickstart]: https://rocket.rs/v0.5/guide/quickstart
//! [getting started]: https://rocket.rs/v0.5/guide/getting-started
//!
//! ## Usage
//!
//! Depend on `rocket` in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rocket = "0.5.1"
//! ```
//!
//! <small>Note that development versions, tagged with `-dev`, are not published
//! and need to be specified as [git dependencies].</small>
//!
//! See the [guide](https://rocket.rs/v0.5/guide) for more information on how
//! to write Rocket applications. Here's a simple example to get you started:
//!
//! [git dependencies]: https://doc.rust-lang.org/cargo/reference/specifying-dependencies.html#specifying-dependencies-from-git-repositories
//!
//! ```rust,no_run
//! #[macro_use] extern crate rocket;
//!
//! #[get("/")]
//! fn hello() -> &'static str {
//!     "Hello, world!"
//! }
//!
//! #[launch]
//! fn rocket() -> _ {
//!     rocket::build().mount("/", routes![hello])
//! }
//! ```
//!
//! ## Features
//!
//! To avoid compiling unused dependencies, Rocket gates certain features. With
//! the exception of `http2`, all are disabled by default:
//!
//! | Feature   | Description                                             |
//! |-----------|---------------------------------------------------------|
//! | `secrets` | Support for authenticated, encrypted [private cookies]. |
//! | `tls`     | Support for [TLS] encrypted connections.                |
//! | `mtls`    | Support for verified clients via [mutual TLS].          |
//! | `http2`   | Support for HTTP/2 (enabled by default).                |
//! | `json`    | Support for [JSON (de)serialization].                   |
//! | `msgpack` | Support for [MessagePack (de)serialization].            |
//! | `uuid`    | Support for [UUID value parsing and (de)serialization]. |
//!
//! Disabled features can be selectively enabled in `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rocket = { version = "0.5.1", features = ["secrets", "tls", "json"] }
//! ```
//!
//! Conversely, HTTP/2 can be disabled:
//!
//! ```toml
//! [dependencies]
//! rocket = { version = "0.5.1", default-features = false }
//! ```
//!
//! [JSON (de)serialization]: crate::serde::json
//! [MessagePack (de)serialization]: crate::serde::msgpack
//! [UUID value parsing and (de)serialization]: crate::serde::uuid
//! [private cookies]: https://rocket.rs/v0.5/guide/requests/#private-cookies
//! [TLS]: https://rocket.rs/v0.5/guide/configuration/#tls
//! [mutual TLS]: crate::mtls
//!
//! ## Configuration
//!
//! Rocket offers a rich, extensible configuration system built on [Figment]. By
//! default, Rocket applications are configured via a `Rocket.toml` file
//! and/or `ROCKET_{PARAM}` environment variables, but applications may
//! configure their own sources. See the [configuration guide] for full details.
//!
//! ## Testing
//!
//! The [`local`] module contains structures that facilitate unit and
//! integration testing of a Rocket application. The top-level [`local`] module
//! documentation and the [testing guide] include detailed examples.
//!
//! [configuration guide]: https://rocket.rs/v0.5/guide/configuration/
//! [testing guide]: https://rocket.rs/v0.5/guide/testing/#testing
//! [Figment]: https://docs.rs/figment

pub mod codegen;
pub mod config;
pub mod handler;
pub mod route;
pub mod server;
