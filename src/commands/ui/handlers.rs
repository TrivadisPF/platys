//! Axum route handlers for the UI server.

use axum::Json;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{Html, IntoResponse, Response};
use super::AppState;
use super::dto::{
    GenerateRequest, GenerateResponse, PropertyDto, ServiceDto, ServicesResponse, PreviewResponse, json_to_yaml,
    yaml_to_json,
};
use crate::config::add_root_indent;
pub(crate) async fn index() -> Html<&'static str> {
    Html(include_str!("assets/index.html"))
}

/// Serves the vendored + app static assets, packaged into the binary at compile time.
pub(crate) async fn asset(Path(file): Path<String>) -> Response {
    if file == "platys.png" {
        let body = include_bytes!("assets/platys.png").as_ref();
        return ([(header::CONTENT_TYPE, "image/png")], body).into_response();
    }
    let (body, content_type) = match file.as_str() {
        "bulma.min.css" => (include_str!("assets/bulma.min.css"), "text/css"),
        "alpine.min.js" => (include_str!("assets/alpine.min.js"), "text/javascript"),
        "app.css" => (include_str!("assets/app.css"), "text/css"),
        "app.js" => (include_str!("assets/app.js"), "text/javascript"),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    ([(header::CONTENT_TYPE, content_type)], body).into_response()
}

///api/services
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
                tags: meta.map(|m| m.tags.clone()).unwrap_or_default(),
                dependencies: meta.map(|m| m.dependencies.clone()).unwrap_or_default(),
                properties: svc
                    .properties
                    .iter()
                    .map(|(k, v)| {
                        let full_property_key = format!("{name}_{k}");
                        let docs = state.docs.properties.get(&full_property_key);
                        let docs_default = docs.and_then(|d| d.default.as_ref());
                        PropertyDto {
                            key: k.clone(),
                            value: yaml_to_json(v),
                            is_bool: matches!(v, yaml_serde::Value::Bool(_))
                                || matches!(docs_default, Some(yaml_serde::Value::Bool(_))),
                            // doc is Option<&PropertyEntry> and description is itself Option<String>,
                            // use and_then to flatten it (instead of Option<Option<String>>)
                            description: docs.and_then(|d| d.description.clone()),
                            allowed_values: docs.and_then(|d| d.allowed_values.clone()),
                            sensitive: docs.map(|d| d.sensitive).unwrap_or(false),
                            default: docs_default.map(yaml_to_json),
                        }
                    })
                    .collect(),
            }
        })
        .collect();

    Json(ServicesResponse { services })
}

///api/generate
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

    let yaml_str = match crate::config::serialize_config(&config, false, true) {
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
    let written = std::path::absolute(&state.config_file)
        .unwrap_or_else(|_| std::path::PathBuf::from(&state.config_file));
    match std::fs::write(&state.config_file, indented) {
        Ok(_) => (
            StatusCode::OK,
            Json(GenerateResponse {
                success: true,
                message: format!("Config written to {}", written.display()),
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

pub(crate) async fn api_preview(
    State(state): State<AppState>,
    Json(req): Json<GenerateRequest>,
) -> (StatusCode, Json<PreviewResponse>) {
    let mut config = state.config.read().await.clone();
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
            config.platys.platform_name = name.clone();
        }
    }
    match crate::config::serialize_config(&config, false, true) {
        Ok(yaml_str) => (StatusCode::OK, Json(PreviewResponse { yaml: yaml_str })),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(PreviewResponse { yaml: format!("# Error: {e}") }),
        ),
    }
}

