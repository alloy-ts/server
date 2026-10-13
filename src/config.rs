//! NAPI bindings for Rocket's server [`Config`] (`rocket::config::Config`).
//!
//! This mirrors the documented `Config` API surface so Node.js callers can
//! build, inspect, and mutate the configuration that drives a Rocket server.
//!
//! The NAPI `Config` class is a faithful, JS-friendly mirror of Rocket's
//! `Config`: every documented field is exposed as a getter/setter, and the
//! static defaults / provider helpers (`debug_default`, `release_default`,
//! `figment`, `try_from`, `from`) are provided as constructors and methods.
//!
//! Composite values (limits, shutdown, TLS) are surfaced as nested NAPI
//! objects (`Limits`, `Shutdown`, `TlsConfig`). The `secret_key` and `tls`
//! fields are gated behind the `secrets` and `tls` crate features
//! respectively; enable them in `Cargo.toml`
//! (`rocket = { features = ["tls", "secrets"] }`) to surface those bindings.

use std::collections::HashMap;
use std::str::FromStr;

use napi::bindgen_prelude::Error as NapiError;
use napi::Status;
use napi_derive::napi;

use rocket::config::{
    Config as RocketConfig, Ident, LogLevel, Shutdown as RocketShutdownAlias, Sig,
};
use rocket::data::{ByteUnit, Limits as RocketLimits};
use rocket::figment::{Profile, value::magic::RelativePathBuf};
use rocket::http::uncased::Uncased;

#[cfg(feature = "secrets")]
use rocket::config::SecretKey;

#[cfg(feature = "tls")]
use rocket::config::{MutualTls as RocketMutualTls, TlsConfig as RocketTlsConfig};

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

// ---------------------------------------------------------------------------
// Associated constants (mirrors `Config`'s stringy parameter names / profiles)
// ---------------------------------------------------------------------------

#[napi]
pub const DEBUG_PROFILE: &'static str = "debug";
#[napi]
pub const RELEASE_PROFILE: &'static str = "release";
#[napi]
pub const DEFAULT_PROFILE: &'static str = "debug";

#[napi]
pub const ADDRESS: &'static str = "address";
#[napi]
pub const PORT: &'static str = "port";
#[napi]
pub const WORKERS: &'static str = "workers";
#[napi]
pub const MAX_BLOCKING: &'static str = "max_blocking";
#[napi]
pub const KEEP_ALIVE: &'static str = "keep_alive";
#[napi]
pub const IDENT: &'static str = "ident";
#[napi]
pub const IP_HEADER: &'static str = "ip_header";
#[napi]
pub const LIMITS: &'static str = "limits";
#[napi]
pub const TLS: &'static str = "tls";
#[napi]
pub const SECRET_KEY: &'static str = "secret_key";
#[napi]
pub const TEMP_DIR: &'static str = "temp_dir";
#[napi]
pub const LOG_LEVEL: &'static str = "log_level";
#[napi]
pub const SHUTDOWN: &'static str = "shutdown";
#[napi]
pub const CLI_COLORS: &'static str = "cli_colors";

// ---------------------------------------------------------------------------
// Composite NAPI value types
// ---------------------------------------------------------------------------

/// Mirrors `rocket::data::Limits`.
///
/// Each entry maps a data-type name (e.g. `"form"`, `"json"`) to a size
/// expression accepted by `ByteUnit`: an integer number of bytes (`"32768"`)
/// or an SI unit string (`"32KiB"`).
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Limits {
    pub limits: HashMap<String, String>,
}

/// Mirrors `rocket::config::Shutdown`.
///
/// All fields are optional so a partial override can be supplied; omitted
/// fields fall back to Rocket's `Shutdown::default()`. Signal names use
/// Rocket's lowercase serde spelling (`"term"`, `"hup"`, ...).
#[napi(object)]
#[derive(Clone, Debug)]
pub struct Shutdown {
    pub ctrlc: Option<bool>,
    pub signals: Option<Vec<String>>,
    pub grace: Option<u32>,
    pub mercy: Option<u32>,
    pub force: Option<bool>,
}

/// Mirrors `rocket::config::TlsConfig`.
#[cfg(feature = "tls")]
#[napi(object)]
#[derive(Clone, Debug)]
pub struct TlsConfig {
    pub certs: String,
    pub key: String,
    pub prefer_server_cipher_order: Option<bool>,
    pub ciphers: Option<Vec<String>>,
    pub mutual: Option<MutualTls>,
}

/// Mirrors the `tls.mutual` portion of `rocket::config::TlsConfig`.
#[cfg(feature = "tls")]
#[napi(object)]
#[derive(Clone, Debug)]
pub struct MutualTls {
    pub ca_certs: String,
    pub mandatory: Option<bool>,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn parse_log_level(s: &str) -> napi::Result<LogLevel> {
    LogLevel::from_str(s).map_err(err)
}

fn parse_ident(value: Option<String>) -> napi::Result<Ident> {
    match value {
        None => Ok(Ident::none()),
        Some(s) => Ident::try_new(&s).map_err(err),
    }
}

fn parse_sig(s: &str) -> napi::Result<Sig> {
    use Sig::*;
    Ok(match s.to_ascii_lowercase().as_str() {
        "alrm" => Alrm,
        "chld" => Chld,
        "hup" => Hup,
        "int" => Int,
        "io" => Io,
        "pipe" => Pipe,
        "quit" => Quit,
        "term" => Term,
        "usr1" => Usr1,
        "usr2" => Usr2,
        other => {
            return Err(NapiError::new(
                Status::InvalidArg,
                format!("invalid signal: {other} (expected one of alrm, chld, hup, int, io, pipe, quit, term, usr1, usr2)"),
            ))
        }
    })
}

fn build_limits(napi: &Limits) -> napi::Result<RocketLimits> {
    let mut limits = RocketLimits::default();
    for (name, raw) in &napi.limits {
        let unit = match ByteUnit::from_str(raw) {
            Ok(u) => u,
            Err(_) => raw
                .parse::<u64>()
                .map(ByteUnit::from)
                .map_err(|_| {
                    NapiError::new(
                        Status::InvalidArg,
                        format!("invalid limit for `{name}`: {raw}"),
                    )
                })?,
        };
        limits = limits.limit(Uncased::from(name.clone()), unit);
    }
    Ok(limits)
}

fn build_shutdown(napi: &Shutdown) -> napi::Result<RocketShutdownAlias> {
    let mut shutdown = RocketShutdownAlias::default();
    if let Some(v) = napi.ctrlc {
        shutdown.ctrlc = v;
    }
    if let Some(sigs) = &napi.signals {
        let mut parsed = std::collections::HashSet::with_capacity(sigs.len());
        for s in sigs {
            parsed.insert(parse_sig(s)?);
        }
        shutdown.signals = parsed;
    }
    if let Some(v) = napi.grace {
        shutdown.grace = v;
    }
    if let Some(v) = napi.mercy {
        shutdown.mercy = v;
    }
    if let Some(v) = napi.force {
        shutdown.force = v;
    }
    Ok(shutdown)
}

#[cfg(feature = "tls")]
fn build_tls(napi: &TlsConfig) -> napi::Result<RocketTlsConfig> {
    let mut tls = RocketTlsConfig::from_paths(napi.certs.clone(), napi.key.clone());
    if let Some(v) = napi.prefer_server_cipher_order {
        tls = tls.with_preferred_server_cipher_order(v);
    }
    if let Some(ciphers) = &napi.ciphers {
        let mut parsed = Vec::with_capacity(ciphers.len());
        for c in ciphers {
            parsed.push(
                serde_json::from_value(serde_json::Value::String(c.clone())).map_err(|e| {
                    NapiError::new(
                        Status::InvalidArg,
                        format!("invalid cipher suite `{c}`: {e}"),
                    )
                })?,
            );
        }
        tls = tls.with_ciphers(parsed);
    }
    if let Some(m) = &napi.mutual {
        let mut mtls = RocketMutualTls::from_path(m.ca_certs.clone());
        if let Some(mand) = m.mandatory {
            mtls = mtls.mandatory(mand);
        }
        tls = tls.with_mutual(mtls);
    }
    Ok(tls)
}

// ---------------------------------------------------------------------------
// Config NAPI class
// ---------------------------------------------------------------------------

#[napi]
pub struct Config {
    profile: Option<String>,
    address: String,
    port: u16,
    workers: u32,
    max_blocking: u32,
    ident: Option<String>,
    ip_header: Option<String>,
    limits: Limits,
    temp_dir: String,
    keep_alive: u32,
    #[cfg(feature = "tls")]
    tls: Option<TlsConfig>,
    #[cfg(feature = "secrets")]
    secret_key: Option<String>,
    shutdown: Shutdown,
    log_level: String,
    cli_colors: bool,
}

#[napi]
impl Config {
    /// Create a `Config` seeded with Rocket's `debug` defaults.
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::from_rocket(&RocketConfig::debug_default())
    }

    // -- static factories -------------------------------------------------

    /// Returns the default configuration for the debug profile.
    #[napi]
    pub fn debug_default() -> Self {
        Self::from_rocket(&RocketConfig::debug_default())
    }

    /// Returns the default configuration for the release profile.
    #[napi]
    pub fn release_default() -> Self {
        Self::from_rocket(&RocketConfig::release_default())
    }

    /// Returns the default configuration for the active profile.
    #[napi]
    pub fn default_config() -> Self {
        Self::from_rocket(&RocketConfig::default())
    }

    /// Returns the configuration extracted from Rocket's default figment
    /// (`Config::figment()`), i.e. defaults merged with `Rocket.toml` and
    /// `ROCKET_*` environment variables.
    #[napi]
    pub fn figment() -> napi::Result<Self> {
        let cfg = RocketConfig::figment()
            .extract::<RocketConfig>()
            .map_err(err)?;
        Ok(Self::from_rocket(&cfg))
    }

    /// Builds a `Config` from a partial override object, mirroring
    /// `Config::try_from(provider)` where the provider is a set of overrides
    /// merged onto the debug defaults.
    #[napi]
    pub fn try_from(init: ConfigInit) -> napi::Result<Self> {
        let mut cfg = Self::debug_default();
        cfg.apply(init)?;
        Ok(cfg)
    }

    /// Builds a `Config` from a partial override object, panicking on error.
    /// Mirrors `Config::from(provider)`.
    #[napi]
    pub fn from(init: ConfigInit) -> Self {
        Self::try_from(init).expect("invalid Config overrides")
    }

    /// All stringy parameter names.
    #[napi]
    pub fn parameters() -> Vec<String> {
        vec![
            ADDRESS.to_string(),
            PORT.to_string(),
            WORKERS.to_string(),
            MAX_BLOCKING.to_string(),
            KEEP_ALIVE.to_string(),
            IDENT.to_string(),
            IP_HEADER.to_string(),
            LIMITS.to_string(),
            TLS.to_string(),
            SECRET_KEY.to_string(),
            TEMP_DIR.to_string(),
            LOG_LEVEL.to_string(),
            SHUTDOWN.to_string(),
            CLI_COLORS.to_string(),
        ]
    }

    // -- runtime helpers --------------------------------------------------

    /// Returns the underlying `rocket::Config` built from the current fields.
    pub(crate) fn build(&self) -> napi::Result<RocketConfig> {
        let mut cfg = RocketConfig::default();
        if let Some(p) = &self.profile {
            cfg.profile = Profile::new(p);
        }
        cfg.address = self
            .address
            .parse()
            .map_err(|e: std::net::AddrParseError| err(e))?;
        cfg.port = self.port;
        cfg.workers = self.workers as usize;
        cfg.max_blocking = self.max_blocking as usize;
        cfg.ident = parse_ident(self.ident.clone())?;
        cfg.ip_header = self.ip_header.clone().map(Uncased::from);
        cfg.limits = build_limits(&self.limits)?;
        cfg.temp_dir = RelativePathBuf::from(std::path::PathBuf::from(self.temp_dir.clone()));
        cfg.keep_alive = self.keep_alive;
        cfg.shutdown = build_shutdown(&self.shutdown)?;
        cfg.log_level = parse_log_level(&self.log_level)?;
        cfg.cli_colors = self.cli_colors;

        #[cfg(feature = "tls")]
        {
            cfg.tls = self.tls.as_ref().map(build_tls).transpose()?;
        }
        #[cfg(feature = "secrets")]
        if let Some(s) = &self.secret_key {
            cfg.secret_key =
                serde_json::from_value(serde_json::Value::String(s.clone())).map_err(|e| {
                    NapiError::new(Status::InvalidArg, format!("invalid secret_key: {e}"))
                })?;
        }

        Ok(cfg)
    }

    /// Returns `true` if TLS is enabled (requires the `tls` feature and a
    /// configured ciphersuite).
    #[cfg(feature = "tls")]
    #[napi]
    pub fn tls_enabled(&self) -> bool {
        self.build().map(|c| c.tls_enabled()).unwrap_or(false)
    }

    /// Returns `true` if mTLS is enabled (requires the `mtls` feature).
    #[cfg(feature = "tls")]
    #[napi]
    pub fn mtls_enabled(&self) -> bool {
        self.build().map(|c| c.mtls_enabled()).unwrap_or(false)
    }

    // -- getters / setters ------------------------------------------------

    #[napi(getter)]
    pub fn profile(&self) -> Option<String> {
        self.profile.clone()
    }
    #[napi(setter)]
    pub fn set_profile(&mut self, value: Option<String>) {
        self.profile = value;
    }

    #[napi(getter)]
    pub fn address(&self) -> String {
        self.address.clone()
    }
    #[napi(setter)]
    pub fn set_address(&mut self, value: String) -> napi::Result<()> {
        value.parse::<std::net::IpAddr>().map_err(err)?;
        self.address = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn port(&self) -> u16 {
        self.port
    }
    #[napi(setter)]
    pub fn set_port(&mut self, value: u16) {
        self.port = value;
    }

    #[napi(getter)]
    pub fn workers(&self) -> u32 {
        self.workers
    }
    #[napi(setter)]
    pub fn set_workers(&mut self, value: u32) {
        self.workers = value;
    }

    #[napi(getter)]
    pub fn max_blocking(&self) -> u32 {
        self.max_blocking
    }
    #[napi(setter)]
    pub fn set_max_blocking(&mut self, value: u32) {
        self.max_blocking = value;
    }

    #[napi(getter)]
    pub fn ident(&self) -> Option<String> {
        self.ident.clone()
    }
    #[napi(setter)]
    pub fn set_ident(&mut self, value: Option<String>) -> napi::Result<()> {
        parse_ident(value.clone())?;
        self.ident = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn ip_header(&self) -> Option<String> {
        self.ip_header.clone()
    }
    #[napi(setter)]
    pub fn set_ip_header(&mut self, value: Option<String>) {
        self.ip_header = value;
    }

    #[napi(getter)]
    pub fn limits(&self) -> Limits {
        self.limits.clone()
    }
    #[napi(setter)]
    pub fn set_limits(&mut self, value: Limits) -> napi::Result<()> {
        build_limits(&value)?; // validate eagerly
        self.limits = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn temp_dir(&self) -> String {
        self.temp_dir.clone()
    }
    #[napi(setter)]
    pub fn set_temp_dir(&mut self, value: String) {
        self.temp_dir = value;
    }

    #[napi(getter)]
    pub fn keep_alive(&self) -> u32 {
        self.keep_alive
    }
    #[napi(setter)]
    pub fn set_keep_alive(&mut self, value: u32) {
        self.keep_alive = value;
    }

    #[cfg(feature = "tls")]
    #[napi(getter)]
    pub fn tls(&self) -> Option<TlsConfig> {
        self.tls.clone()
    }
    #[cfg(feature = "tls")]
    #[napi(setter)]
    pub fn set_tls(&mut self, value: Option<TlsConfig>) -> napi::Result<()> {
        if let Some(t) = &value {
            build_tls(t)?; // validate eagerly
        }
        self.tls = value;
        Ok(())
    }

    #[cfg(feature = "secrets")]
    #[napi(getter)]
    pub fn secret_key(&self) -> Option<String> {
        self.secret_key.clone()
    }
    #[cfg(feature = "secrets")]
    #[napi(setter)]
    pub fn set_secret_key(&mut self, value: Option<String>) -> napi::Result<()> {
        if let Some(s) = &value {
            serde_json::from_value::<SecretKey>(serde_json::Value::String(s.clone()))
                .map_err(|e| NapiError::new(Status::InvalidArg, format!("invalid secret_key: {e}")))?;
        }
        self.secret_key = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn shutdown(&self) -> Shutdown {
        self.shutdown.clone()
    }
    #[napi(setter)]
    pub fn set_shutdown(&mut self, value: Shutdown) -> napi::Result<()> {
        build_shutdown(&value)?; // validate eagerly
        self.shutdown = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn log_level(&self) -> String {
        self.log_level.clone()
    }
    #[napi(setter)]
    pub fn set_log_level(&mut self, value: String) -> napi::Result<()> {
        parse_log_level(&value)?;
        self.log_level = value;
        Ok(())
    }

    #[napi(getter)]
    pub fn cli_colors(&self) -> bool {
        self.cli_colors
    }
    #[napi(setter)]
    pub fn set_cli_colors(&mut self, value: bool) {
        self.cli_colors = value;
    }
}

impl Config {
    /// Build this NAPI mirror from a concrete `rocket::Config`.
    fn from_rocket(cfg: &RocketConfig) -> Self {
        Config {
            profile: Some(cfg.profile.as_str().to_string()),
            address: cfg.address.to_string(),
            port: cfg.port,
            workers: cfg.workers as u32,
            max_blocking: cfg.max_blocking as u32,
            ident: cfg.ident.as_str().map(|s| s.to_string()),
            ip_header: cfg.ip_header.as_ref().map(|u| u.as_str().to_string()),
            // Rocket's `Limits` is not publicly iterable, so revert to empty.
            limits: Limits {
                limits: HashMap::new(),
            },
            temp_dir: cfg.temp_dir.relative().display().to_string(),
            keep_alive: cfg.keep_alive,
            #[cfg(feature = "tls")]
            tls: None,
            #[cfg(feature = "secrets")]
            secret_key: None,
            shutdown: Shutdown {
                ctrlc: Some(cfg.shutdown.ctrlc),
                signals: Some(
                    cfg.shutdown
                        .signals
                        .iter()
                        .map(|s| format!("{s:?}").to_ascii_lowercase())
                        .collect(),
                ),
                grace: Some(cfg.shutdown.grace),
                mercy: Some(cfg.shutdown.mercy),
                force: Some(cfg.shutdown.force),
            },
            log_level: cfg.log_level.to_string(),
            cli_colors: cfg.cli_colors,
        }
    }

    /// Apply a partial `ConfigInit` override onto this config.
    fn apply(&mut self, init: ConfigInit) -> napi::Result<()> {
        if let Some(v) = init.profile {
            self.profile = Some(v);
        }
        if let Some(v) = init.address {
            self.set_address(v)?;
        }
        if let Some(v) = init.port {
            self.port = v;
        }
        if let Some(v) = init.workers {
            self.workers = v;
        }
        if let Some(v) = init.max_blocking {
            self.max_blocking = v;
        }
        if init.ident_none == Some(true) {
            self.ident = None;
        } else if let Some(v) = init.ident {
            self.set_ident(Some(v))?;
        }
        if init.ip_header_none == Some(true) {
            self.ip_header = None;
        } else if let Some(v) = init.ip_header {
            self.ip_header = Some(v);
        }
        if let Some(v) = init.limits {
            self.set_limits(v)?;
        }
        if let Some(v) = init.temp_dir {
            self.temp_dir = v;
        }
        if let Some(v) = init.keep_alive {
            self.keep_alive = v;
        }
        #[cfg(feature = "tls")]
        if let Some(v) = init.tls {
            self.set_tls(Some(v))?;
        }
        #[cfg(feature = "secrets")]
        if let Some(v) = init.secret_key {
            self.set_secret_key(Some(v))?;
        }
        if let Some(v) = init.shutdown {
            self.set_shutdown(v)?;
        }
        if let Some(v) = init.log_level {
            self.set_log_level(v)?;
        }
        if let Some(v) = init.cli_colors {
            self.cli_colors = v;
        }
        Ok(())
    }
}

/// Partial override object accepted by [`Config::try_from`] / [`Config::from`].
///
/// Every field is optional; supplied values override the debug defaults,
/// mirroring `Config::try_from(figment.merge(...))`.
///
/// `identNone` / `ipHeaderNone` force the corresponding value to `None`
/// (server identity or client-IP header disabled), since `null` alone cannot
/// distinguish "unset" from "disable" for these `Option` fields.
#[napi(object)]
pub struct ConfigInit {
    pub profile: Option<String>,
    pub address: Option<String>,
    pub port: Option<u16>,
    pub workers: Option<u32>,
    pub max_blocking: Option<u32>,
    pub ident: Option<String>,
    pub ident_none: Option<bool>,
    pub ip_header: Option<String>,
    pub ip_header_none: Option<bool>,
    pub limits: Option<Limits>,
    pub temp_dir: Option<String>,
    pub keep_alive: Option<u32>,
    #[cfg(feature = "tls")]
    pub tls: Option<TlsConfig>,
    #[cfg(feature = "secrets")]
    pub secret_key: Option<String>,
    pub shutdown: Option<Shutdown>,
    pub log_level: Option<String>,
    pub cli_colors: Option<bool>,
}
