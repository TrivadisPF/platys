//! Request/response shapes for the UI HTTP API, plus the helpers that
//! convert between `serde_yaml` and `serde_json` values.



#[derive(serde::Serialize)]
pub(crate) struct ServiceDto {
    pub(crate) name: String, // config name ex: KAFKA
    pub(crate) display_name: String, // friendly name, if absent fallback to `name
    pub(crate) description: String,
    pub(crate) category: String,
    pub(crate) enabled: bool,
    pub(crate) properties: Vec<PropertyDto>,

}


#[derive(serde::Serialize)]
pub(crate) struct PropertyDto {
    pub(crate) key: String,
    pub(crate) value: serde_json::Value,
    pub(crate) description: Option<String>,
    pub(crate) allowed_values: Option<Vec<String>>,
    pub(crate) sensitive: bool,
    pub(crate) default: Option<serde_json::Value>,
}

/// What `GET /api/services` returns.
#[derive(serde::Serialize)]
pub(crate) struct ServicesResponse {
    pub(crate) services: Vec<ServiceDto>,
}

// ── Value conversion helpers ────────────────────────────────────────────────

/// Convert a `serde_yaml::Value` to a `serde_json::Value` for the API response.
pub(crate) fn yaml_to_json(v: &yaml_serde::Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

/// Convert a `serde_json::Value` from the API request to a `serde_yaml::Value`.
pub(crate) fn json_to_yaml(v: &serde_json::Value) -> yaml_serde::Value {
    yaml_serde::to_value(v).unwrap_or(yaml_serde::Value::Null)
}


#[derive(serde::Deserialize)]
pub(crate) struct ServiceUpdate {
    pub(crate) name: String,
    pub(crate) enabled: bool,
    pub(crate) properties: std::collections::HashMap<String, serde_json::Value>,
}

/// What `POST /api/generate` receives.
#[derive(serde::Deserialize)]
pub(crate) struct GenerateRequest {
    pub(crate) services: Vec<ServiceUpdate>,
    pub(crate) platform_name: Option<String>,
}

/// What `POST /api/generate` returns.
#[derive(serde::Serialize)]
pub(crate) struct GenerateResponse {
    pub(crate) success: bool,
    pub(crate) message: String,
}
