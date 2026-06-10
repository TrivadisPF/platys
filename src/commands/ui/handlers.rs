//! Axum route handlers for the UI server.

use super::AppState;
use super::dto::{
    GenerateRequest, GenerateResponse, PropertyDto, ServiceDto, ServicesResponse, json_to_yaml,
    yaml_to_json,
};
use crate::config::add_root_indent;
use axum::extract::State;
use axum::http::StatusCode;
use axum::{Json, response::Html};

pub(crate) async fn index() -> Html<&'static str> {
    Html(
        r#"<!DOCTYPE html>
  <html>
  <head>
      <title>platys UI</title>
      <style>
          body { font-family: system-ui, sans-serif; padding: 2em; }
          h1 { color: #c8102e; }
      </style>
  </head>
  <body>
      <h1>platys UI</h1>
      <p>The server is running. The real UI will live here in later phases.</p>
  </body>
  </html>"#,
    )
}

pub(crate) async fn api_services(State(state): State<AppState>) -> Json<ServicesResponse> {
    let config = state.config.read().await;

    let services = config
        .services
        .iter()
        .map(|(name, svc)| {
            let meta = state.docs.services.get(name);
            ServiceDto {
                name: name.clone(),
                display_name: meta.map(|m| m.name.clone()).unwrap_or_else(|| name.clone()),
                description: meta.map(|m| m.description.clone()).unwrap_or_default(),
                category: state.docs.categories.get(name).cloned().unwrap_or_default(),
                enabled: svc.enabled,
                properties: svc
                    .properties
                    .iter()
                    .map(|(k, v)| {
                        let docs = state.docs.properties.get(k);
                        PropertyDto {
                            key: k.clone(),
                            value: yaml_to_json(v),
                            // doc is Option<&PropertyEntry> and description is itself Option<String>,
                            // use and_then to flatten it (instead of Option<Option<String>>)
                            description: docs.and_then(|d| d.description.clone()),
                            allowed_values: docs.and_then(|d| d.allowed_values.clone()),
                            sensitive: docs.map(|d| d.sensitive).unwrap_or(false),
                            default: docs
                                .and_then(|d| d.default.as_ref())
                                .map(|default| yaml_to_json(default)),
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    Json(ServicesResponse { services })
}

pub(crate) async fn api_generate(
    State(state): State<AppState>,
    Json(req): Json<GenerateRequest>,
) -> (StatusCode, Json<GenerateResponse>) {
    let mut config = state.config.write().await;
    for update in &req.services {
        if let Some(svc) = config.services.get_mut(&update.name) {
            svc.enabled = update.enabled;
            for (key, val) in &update.properties {
                svc.properties.insert(key.clone(), json_to_yaml(val));
            }
        }
    }

    if let Some(name) = &req.platform_name {
        if !name.is_empty() {
            log::info!("Updating platform with name: {}", name);
            config.platys.platform_name = name.clone();
        }
    }

    let yaml_str = match crate::config::serialize_config(&config) {
        Ok(yaml_str) => yaml_str,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(GenerateResponse {
                    success: false,
                    message: e.to_string(),
                }),
            );
        }
    };

    //finalize file with proper indentation so that it works with the generator
    let indented = add_root_indent(&yaml_str, 6);
    match std::fs::write(&state.config_file, indented) {
        Ok(_) => (
            StatusCode::OK,
            Json(GenerateResponse {
                success: true,
                message: format!("Config written to {}", state.config_file),
            }),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(GenerateResponse {
                success: false,
                message: e.to_string(),
            }),
        ),
    }
}
