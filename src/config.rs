use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;


// ── Indentation helper ────────────────────────────────────────────────────────


//Indentation is needed as this file actually is nerged to an existing one on which the platys
// info needs to be indented
pub fn add_root_indent(yaml: &str, n: usize) -> String {
    let mut out = String::with_capacity(yaml.len() + yaml.lines().count() * n + 1);
    for line in yaml.lines() {
        for _ in 0..n { out.push(' '); }
        out.push_str(line);
        out.push('\n');
    }
    out
}

// ── Validation ────────────────────────────────────────────────────────────────

const LEGACY_PARAMS: &[&str] = &["stack-image-name", "stack-image-version"];

pub fn is_older_version(raw: &str) -> bool {
    LEGACY_PARAMS.iter().any(|p| raw.contains(p))
}

pub fn validate_platys(p: &PlatysSection, raw: &str) -> Result<()> {
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
    let resp = reqwest::blocking::get(url).with_context(|| format!("Failed to GET {url}"))?;

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

fn is_service_name(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_')
}

fn is_property_name(s: &str) -> bool {
    !s.is_empty()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
}

pub fn parse_config(raw: &str) -> Result<ParsedConfig> {
    let root: serde_yaml::Mapping =
        serde_yaml::from_str(raw).context("Failed to parse YAML file")?;
    let mut cfg = ParsedConfig::default();
    let mut current_service: Option<String> = None;

    for (k, v) in root {
        let key = match k.as_str() {
            Some(k) => k.to_string(),
            None => continue,
        };

        //handle platys section
        if key == "platys" {
            cfg.platys = serde_yaml::from_value(v).context("Failed to parse [platys] section")?;
            continue;
        }

        //handle services
        if let Some(svc) = key.strip_suffix("_enable") {
            if is_service_name(svc) {
                let enabled = v.as_bool().unwrap_or(false);
                let entry = cfg.services.entry(svc.to_string()).or_default();
                entry.enabled = enabled;
                current_service = Some(svc.to_string()); //keep track of current service
                continue;
            }
        }

        // handle service properties (if we have a current service)
        if let Some(svc) = &current_service {
            if let Some(property_name) = key
                .strip_prefix(svc.as_str())
                .and_then(|s| s.strip_prefix('_'))
            {
                if is_property_name(property_name) {
                    cfg.services
                        .get_mut(svc)
                        .unwrap()
                        .properties
                        .insert(property_name.to_string(), v);
                    continue;
                }
            }
        }
        cfg.globals.insert(key, v); // not platys, service or property section, fallback into globals
    }

    Ok(cfg)
}

pub fn serialize_config(cfg: &ParsedConfig) -> Result<String> {
    let mut root = serde_yaml::Mapping::new();

    // create platys section
    root.insert(
        Value::String("platys".to_string()),
        serde_yaml::to_value(&cfg.platys).context("Failed to serialize platys")?,
    );

    // create globals in the order they were parsed
    for (k, v) in &cfg.globals {
        root.insert(Value::String(k.clone()), v.clone());
    }

    //create services
    for (svc_name, svc) in &cfg.services {
        root.insert(
            Value::String(format!("{svc_name}_enable")),
            Value::Bool(svc.enabled),
        );

        //append service's properties
        for (property, value) in &svc.properties {
            root.insert(
                Value::String(format!("{svc_name}_{property}")),
                value.clone(),
            );
        }
    }

    serde_yaml::to_string(&root).context("Failed to serialize config")
}

//Structs



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

#[derive(Debug, Default)]
pub struct ParsedConfig {
    pub platys: PlatysSection,
    pub globals: IndexMap<String, Value>,
    pub services: IndexMap<String, Service>,
}

#[derive(Debug, Default)]
pub struct Service {
    pub enabled: bool,
    pub properties: IndexMap<String, Value>,
}

#[cfg(test)]
mod tests{
    use super::*;
    use indoc::indoc;
    //import everything from the module
    //check new random service is properly parsed
    #[test]
    fn parses_arbitrary_new_service() {
        let yaml = indoc! {"
          platys:
            platform-name: test
            platform-stack: test/stack
            platform-stack-version: '1.0'
            structure: flat
          OPENSEARCH_enable: true
          OPENSEARCH_nodes: 5
          OPENSEARCH_replicas: 2
      "};

        let cfg = parse_config(&yaml).expect("Should Parse");
        let svc = cfg.services.get("OPENSEARCH").expect("OPENSEARCH service should exist");
        assert!(svc.enabled);
        assert_eq!(svc.properties.len(), 2);
        assert!(svc.properties.contains_key("nodes"));
        assert!(svc.properties.contains_key("replicas"));
    }


    // --- Test 2: properties whose names contain `_enable` are NOT toggles ---
    #[test]
    fn property_with_enable_in_name_is_property(){
        let yaml = indoc! {"
              platys:
                platform-name: t
                platform-stack: t/s
                platform-stack-version: '1'
                structure: flat
              KAFKA_enable: true
              KAFKA_delete_topic_enable: true
              KAFKA_auto_create_topics_enable: false
          "};

        let cfg = parse_config(&yaml).expect("Should Parse");
        assert_eq!(cfg.services.len(), 1, "expected exactly one service");
        let kafka = cfg.services.get("KAFKA").expect("KAFKA service should exist");
        assert!(kafka.enabled);
        assert!(kafka.properties.contains_key("delete_topic_enable"));
        assert!(kafka.properties.contains_key("auto_create_topics_enable"));
        assert_eq!(kafka.properties["auto_create_topics_enable"].as_bool(), Some(false));
        assert_eq!(kafka.properties["delete_topic_enable"].as_bool(), Some(true));

    }

    // --- Test 3: round-trip text → struct → text → struct preserves content ---
    #[test]
    fn round_trip_perserves_content(){
        let yaml = indoc! {"
              platys:
                platform-name: test
                platform-stack: stack
                platform-stack-version: '1.0'
                structure: flat
              use_timezone: ''
              generate_passwords: false
              KAFKA_enable: true
              KAFKA_broker_nodes: 3
              KAFKA_broker_first_port: 9092
              ZOOKEEPER_enable: false
              ZOOKEEPER_nodes: 1
          "};

        let cfg = parse_config(&yaml).expect("Should Parse");
        let txt = serialize_config(&cfg).expect("Should serialize config");
        let reparsed_cfg = parse_config(&txt).expect("Should Parse");

        assert_eq!(&reparsed_cfg.services.len(), &cfg.services.len());
        assert_eq!(&reparsed_cfg.globals.len(), &cfg.globals.len());
        assert!(&reparsed_cfg.services["KAFKA"].enabled);
        assert!(!&reparsed_cfg.services["ZOOKEEPER"].enabled);
        assert_eq!(&reparsed_cfg.services["KAFKA"].properties.len(), &cfg.services["KAFKA"].properties.len());
    }


    #[test]
    fn add_root_indent_prepends_spaces_to_each_line() {
        let input = "foo\nbar\n";
        let out = add_root_indent(input, 4);
        assert_eq!(out, "    foo\n    bar\n");
    }
}
