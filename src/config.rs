use anyhow::{ bail, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};

// ── Regex for service key detection ──────────────────────────────────────────

pub const SERVICE_REGEX: &str = r"^([A-Z0-9]+(?:_[A-Z0-9]+)*)_enable$";

pub fn service_regex() -> Regex {
    Regex::new(SERVICE_REGEX).expect("SERVICE_REGEX is valid")
}

// ── Typed config structs (used by `gen`) ─────────────────────────────────────

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct YamlFile {
    pub platys: PlatysSection,
}

#[derive(Debug, Default, Deserialize, Serialize)]
pub struct PlatysSection {
    #[serde(rename = "platform-name", default)]
    pub platform_name: String,
    #[serde(rename = "platform-stack", default)]
    pub platform_stack: String,
    #[serde(rename = "platform-stack-version", default)]
    pub platform_stack_version: String,
    #[serde(default)]
    pub structure: String,
}

// ── Node-level YAML helpers (used by `init`) ─────────────────────────────────

/// Returns (true, service_name) if the YAML key matches the service-enable pattern.
pub fn is_service_key(key: &str) -> (bool, String) {
    let re = service_regex();
    if let Some(caps) = re.captures(key) {
        (true, caps[1].to_string())
    } else {
        (false, String::new())
    }
}

/// Returns true if `key` looks like a property of `service` (e.g. KAFKA_foo_bar)
/// but is NOT itself an `_enable` key.
pub fn is_service_property(service: &str, key: &str) -> bool {
    let enable_re = service_regex();
    if enable_re.is_match(key) {
        return false; // it IS an enable key, not a property
    }
    let escaped = regex::escape(service);
    let pattern = format!(r"^{escaped}_[a-z0-9_]+$");
    Regex::new(&pattern)
        .expect("dynamic pattern is valid")
        .is_match(key)
}

// ── Indentation helper ────────────────────────────────────────────────────────


pub fn add_root_indent(yaml: &str, n: usize) -> String {
    let prefix = " ".repeat(n);
    yaml.lines()
        .map(|l| format!("{prefix}{l}"))
        .collect::<Vec<_>>()
        .join("\n")
        + "\n"
}

// ── Validation ────────────────────────────────────────────────────────────────

const LEGACY_PARAMS: &[&str] = &["stack-image-name", "stack-image-version"];

pub fn is_older_version(raw: &str) -> bool {
    LEGACY_PARAMS.iter().any(|p| raw.contains(p))
}

pub fn validate_platys(cfg: &YamlFile, raw: &str) -> Result<()> {
    let p = &cfg.platys;
    if p.platform_name.is_empty()
        || p.platform_stack.is_empty()
        || p.platform_stack_version.is_empty()
        || p.structure.is_empty()
    {
        if is_older_version(raw) {
            bail!(
                "The config file uses legacy key names [stack-image-name / stack-image-version].\n\
                 Please rename them: stack-image-name → platform-stack, \
                 stack-image-version → platform-stack-version"
            );
        } else {
            bail!(
                "Config file is missing required fields. \
                 Ensure [platform-name], [platform-stack], [platform-stack-version], \
                 and [structure] are set."
            );
        }
    }

    if p.structure != "subfolder" && p.structure != "flat" {
        bail!(
            "Invalid [structure] value '{}'. Accepted values: flat | subfolder",
            p.structure
        );
    }

    Ok(())
}

// ── Node-limit map (used by `gen`) ────────────────────────────────────────────

pub fn node_limit(name: &str) -> Option<u32> {
    match name {
        "ZOOKEEPER_nodes" => Some(3),
        "KAFKA_broker_nodes" => Some(6),
        "KAFKA_SCHEMA_REGISTRY_nodes" => Some(2),
        "KAFKA_CONNECT_nodes" => Some(3),
        "KAFKA_KSQLDB_nodes" => Some(3),
        "HADOOP_datanodes" => Some(6),
        "DATASTAX_nodes" => Some(3),
        "MOSQUITTO_nodes" => Some(3),
        _ => None,
    }
}

// ── Remote file download ──────────────────────────────────────────────────────

pub fn download_remote_file(url: &str) -> Result<std::path::PathBuf> {
    let resp = reqwest::blocking::get(url)
        .with_context(|| format!("Failed to GET {url}"))?;

    let mut tmp = tempfile::Builder::new()
        .prefix("config")
        .suffix(".yml")
        .tempfile()
        .context("Failed to create temp file")?;

    let bytes = resp.bytes().context("Failed to read response bytes")?;
    std::io::Write::write_all(&mut tmp, &bytes).context("Failed to write temp file")?;

    let (_, path) = tmp.keep().context("Failed to persist temp file")?;
    Ok(path)
}

// ── Banner ────────────────────────────────────────────────────────────────────

pub fn print_banner(path: &str) {
    // Embedded at compile time from assets/init_banner.txt
    // (mirrors Go's embed.FS usage)
    const BANNER: &str = include_str!("../assets/init_banner.txt");
    println!("{}", BANNER.replace("{}", path));
}
