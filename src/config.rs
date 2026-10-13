//! NAPI bindings for Rocket's server [`Config`] (`rocket::config::Config`).
//!
//! This mirrors the documented `Config` API surface so Node.js callers can
//! build, inspect, and mutate the configuration that drives a Rocket server.

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

fn err<E: std::fmt::Display>(e: E) -> NapiError {
    NapiError::new(Status::GenericFailure, e.to_string())
}

// ---------------------------------------------------------------------------
// Associated constants
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
#[napi(object)]
#[derive(Clone)]
pub struct Limits {
    pub limits: HashMap<String, String>,
}

/// Mirrors `rocket::config::Shutdown`.
#[napi(object)]
#[derive(Clone)]
pub struct Shutdown {
    pub ctrlc: Option<bool>,
    pub signals: Option<Vec<String>>,
    pub grace: Option<u32>,
    pub mercy: Option<u32>,
    pub force: Option<bool>,
}

// ---------------------------------------------------------------------------
// Conversion helpers
// ---------------------------------------------------------------------------

fn parse_log_level(s: &str) -> napi::Result<LogLevel> {
    LogLevel::from_str(s).map_err(err)
}

fn parse_ident(value: Option<&str>) -> napi::Result<Ident> {
    match value {
        None => Ok(Ident::none()),
        Some(s) => Ident::try_new(s).map_err(err),
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
        limits = limits.limit(name.to_string(), unit);
    }
    Ok(limits)
}

fn build_shutdown(napi: &Shutdown) -> napi::Result<RocketShutdownAlias> {
    let mut shutdown = RocketShutdownAlias::default();
    if let Some(v) = napi.ctrlc {
        shutdown.ctrlc = v;
    }
    if let Some(sigs) = &napi.signals {
        let mut parsed = Vec::with_capacity(sigs.len());
        for s in sigs {
            parsed.push(parse_sig(s)?);
        }
        shutdown.signals = parsed.into_iter().collect();
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
    shutdown: Shutdown,
    log_level: String,
    cli_colors: bool,
}

#[napi]
impl Config {
    #[napi(constructor)]
    pub fn new() -> Self {
        Self::from_rocket(&RocketConfig::debug_default())
    }

    #[napi]
    pub fn debug_default() -> Self {
        Self::from_rocket(&RocketConfig::debug_default())
    }

    #[napi]
    pub fn release_default() -> Self {
        Self::from_rocket(&RocketConfig::release_default())
    }

    #[napi]
    pub fn default_config() -> Self {
        Self::from_rocket(&RocketConfig::default())
    }

    #[napi]
    pub fn figment() -> napi::Result<Self> {
        let cfg = RocketConfig::figment()
            .extract::<RocketConfig>()
            .map_err(err)?;
        Ok(Self::from_rocket(&cfg))
    }

    #[napi]
    pub fn try_from(init: ConfigInit) -> napi::Result<Self> {
        let mut cfg = Self::debug_default();
        cfg.apply(init)?;
        Ok(cfg)
    }

    #[napi]
    pub fn from(init: ConfigInit) -> Self {
        Self::try_from(init).expect("invalid Config overrides")
    }

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
        cfg.ident = parse_ident(self.ident.as_deref())?;
        cfg.ip_header = self.ip_header.clone().map(Uncased::from);
        cfg.limits = build_limits(&self.limits)?;
        cfg.temp_dir = RelativePathBuf::from(self.temp_dir.clone());
        cfg.keep_alive = self.keep_alive;
        cfg.shutdown = build_shutdown(&self.shutdown)?;
        cfg.log_level = parse_log_level(&self.log_level)?;
        cfg.cli_colors = self.cli_colors;

        Ok(cfg)
    }

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
        parse_ident(value.as_deref())?;
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
        build_limits(&value)?;
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

    #[napi(getter)]
    pub fn shutdown(&self) -> Shutdown {
        self.shutdown.clone()
    }
    #[napi(setter)]
    pub fn set_shutdown(&mut self, value: Shutdown) -> napi::Result<()> {
        build_shutdown(&value)?;
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
    fn from_rocket(cfg: &RocketConfig) -> Self {
        Config {
            profile: Some(cfg.profile.as_str().to_string()),
            address: cfg.address.to_string(),
            port: cfg.port,
            workers: cfg.workers as u32,
            max_blocking: cfg.max_blocking as u32,
            ident: cfg.ident.as_str().map(|s| s.to_string()),
            ip_header: cfg.ip_header.as_ref().map(|u| u.as_str().to_string()),
            limits: Limits {
                limits: HashMap::new(),
            },
            temp_dir: cfg.temp_dir.relative().to_string_lossy().to_string(),
            keep_alive: cfg.keep_alive,
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
    pub shutdown: Option<Shutdown>,
    pub log_level: Option<String>,
    pub cli_colors: Option<bool>,
}
