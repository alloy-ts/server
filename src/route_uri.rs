use napi::bindgen_prelude::*;
use napi_derive::napi;

/// NAPI wrapper for Rocket's RouteUri.
#[napi]
pub struct RouteUri {
    base_str: String,
    unmounted_origin_str: String,
    origin_str: String,
    path_str: String,
    query_str: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Color {
    Static = 3,
    Partial = 2,
    Wild = 1,
}

#[napi]
impl RouteUri {
    /// Constructs a new RouteUri given a base mount point and route URI string.
    #[napi(constructor)]
    pub fn new(base: Option<String>, uri: String) -> Result<Self> {
        let base_val = base.unwrap_or_else(|| "/".to_string());

        let clean_base = if base_val.starts_with('/') {
            base_val
        } else {
            format!("/{}", base_val)
        };

        let clean_uri = if uri.starts_with('/') {
            uri
        } else {
            format!("/{}", uri)
        };

        let (path_part, query_part) = match clean_uri.find('?') {
            Some(idx) => {
                let (p, q) = clean_uri.split_at(idx);
                let q_trimmed = &q[1..];
                (p.to_string(), if q_trimmed.is_empty() { None } else { Some(q_trimmed.to_string()) })
            }
            None => (clean_uri.clone(), None),
        };

        let unmounted_origin = clean_uri;

        let origin_path = if clean_base == "/" {
            path_part.clone()
        } else {
            format!("{}{}", clean_base.trim_end_matches('/'), path_part)
        };

        let full_origin = match &query_part {
            Some(q) => format!("{}?{}", origin_path, q),
            None => origin_path.clone(),
        };

        Ok(Self {
            base_str: clean_base,
            unmounted_origin_str: unmounted_origin,
            origin_str: full_origin,
            path_str: origin_path,
            query_str: query_part,
        })
    }

    /// The path of the base mount point of this route URI.
    #[napi(getter)]
    pub fn base(&self) -> String {
        self.base_str.clone()
    }

    /// The unmounted origin URI (without base).
    #[napi(getter)]
    pub fn unmounted_origin(&self) -> String {
        self.unmounted_origin_str.clone()
    }

    /// The canonical route origin URI (including base).
    #[napi(getter)]
    pub fn origin(&self) -> String {
        self.origin_str.clone()
    }

    /// The path portion of the canonical route URI.
    #[napi(getter)]
    pub fn path(&self) -> String {
        self.path_str.clone()
    }

    /// The query portion of the canonical route URI, if present.
    #[napi(getter)]
    pub fn query(&self) -> Option<String> {
        self.query_str.clone()
    }

    /// The full URI as a string.
    #[napi(getter)]
    pub fn as_str(&self) -> String {
        self.origin_str.clone()
    }

    /// Returns default rank computed for this route URI according to Rocket's 12-tier ranking table.
    #[napi(getter)]
    pub fn default_rank(&self) -> i32 {
        let path_segs: Vec<&str> = self
            .path_str
            .split('/')
            .filter(|s| !s.is_empty())
            .collect();

        let static_path = path_segs.iter().all(|s| !s.contains('<') && !s.contains(':'));
        let wild_path = !path_segs.is_empty() && path_segs.iter().all(|s| s.contains('<') || s.contains(':'));

        let path_color = match (static_path, wild_path) {
            (true, _) => Color::Static,
            (_, true) => Color::Wild,
            (_, _) => Color::Partial,
        };

        let query_color = self.query_str.as_ref().map(|q| {
            let query_segs: Vec<&str> = q.split('&').filter(|s| !s.is_empty()).collect();
            let static_query = query_segs.iter().all(|s| !s.contains('<') && !s.contains(':'));
            let wild_query = query_segs.iter().all(|s| s.contains('<') || s.contains(':'));
            match (static_query, wild_query) {
                (true, _) => Color::Static,
                (_, true) => Color::Wild,
                (_, _) => Color::Partial,
            }
        });

        let raw_path_weight = path_color as i32;
        let raw_query_weight = query_color.map_or(0, |c| c as i32);
        let raw_weight = (raw_path_weight << 2) | raw_query_weight;

        -(raw_weight - 3)
    }

    /// Remaps the base mount point and returns a new RouteUri instance.
    #[napi]
    pub fn map_base(&self, new_base: String) -> Result<Self> {
        Self::new(Some(new_base), self.unmounted_origin_str.clone())
    }
}
