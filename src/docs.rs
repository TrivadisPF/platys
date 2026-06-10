use serde::Deserialize;
use std::collections::HashMap;


// Expected paths inside the Docker image once released.
// For local development, use the --docs-path CLI flag instead.
pub const SERVICES_YML_PATH: &str = "/opt/mdps-gen/vars/services.yml";
pub const INDEX_YML_PATH: &str = "/opt/mdps-gen/vars/index.yml";

/** services.yml Model **/
// Mirrors the top level of services.yml:
//   services:
//     akhq: ...
//     kafka: ...
#[derive(Debug, Deserialize)]
pub struct ServicesYaml {
    pub services: HashMap<String, ServiceEntry>,
}

// One service block, e.g. the `akhq:` section
#[derive(Debug, Deserialize)]
pub struct ServiceEntry {
    pub name: Option<String>, //display name like akhq
    pub description: Option<String>,
    pub enable: Option<EnableSection>, // contains the config key name
    pub properties: Option<HashMap<String, PropertyEntry>>,
}

// The `enable:` block — tells us the config prefix
//   enable:
//     platys_init: AKHQ

#[derive(Debug, Deserialize)]
pub struct EnableSection {
    pub platys_init: Option<String>, // name of the service (Kafka, akhq, etc.)
}

// One property block, e.g. AKHQ_topic_page_size
// #[serde(default)] on the struct means missing fields become Default::default()
#[derive(Debug, Default, Clone, Deserialize)]
pub struct PropertyEntry {
    pub description: Option<String>,
    pub default: Option<yaml_serde::Value>, // the raw YAML default value
    pub allowed_values: Option<Vec<String>>,
    #[serde(default)] // if not present in YAML, defaults to false
    pub sensitive: bool,
    pub applicable_when: Option<String>,
    pub since: Option<String>,
}

/** index.yml Model **/

// Mirrors the top level of index.yml:
//   configuration:
//     - name: Overall Settings
//       source: overall.yml
//     - name: Internal Services
//       sections:
//         - name: Stream Processing
//           services: [kafka, ...]

#[derive(Debug, Deserialize)]
pub struct IndexYaml {
    pub configuration: Vec<ConfigSection>,
}

// A top-level section — either has subsections OR a source file
#[derive(Debug, Deserialize)]
pub struct ConfigSection {
    pub name: String,
    // Some top-level sections just point to another file (e.g. "Overall Settings")
    // Those have no subsections — we skip them when building categories
    pub sections: Option<Vec<CategorySection>>,
}

// A named group of services within a section
#[derive(Debug, Deserialize)]
pub struct CategorySection {
    pub name: String,
    pub description: Option<String>,
    // Slugs like ["kafka", "schema-registry", "kafka-connect"]
    pub services: Option<Vec<String>>,
}

/** inner structs **/

//Holds display name and description for a service
pub struct ServiceMeta {
    pub name: String,
    pub description: String,
}

// The pre-built lookup — stored in AppState at server startup
pub struct PlatysIndex {
    //Ex "KAFKA" -> display_name, description
    pub services: HashMap<String, ServiceMeta>,

    // "AKHQ_topic_page_size" -> property documentation
    pub properties: HashMap<String, PropertyEntry>,

    // "AKHQ" -> "Stream Processing"
    pub categories: HashMap<String, String>,
}

impl PlatysIndex {
    pub fn build(services_raw: &str, index_raw: &str) -> anyhow::Result<Self> {
        use anyhow::Context;
        //parse both YML files into their struct
        let svc_file: ServicesYaml =
            yaml_serde::from_str(services_raw).context("Failed to parse services YAML")?;
        let index_file: IndexYaml =
            yaml_serde::from_str(index_raw).context("Failed to parse index YAML")?;

        let mut services: HashMap<String, ServiceMeta> = HashMap::new();
        let  mut properties: HashMap<String, PropertyEntry> = HashMap::new();



        // We need slug → config name when building categories
        // e.g. "akhq" -> "AKHQ"
        let mut slug_to_config: HashMap<String, String> = HashMap::new();

        for (slug, entry) in svc_file.services {
            // Prefer enable.platys_init for the config name because it's explicit.
            // Fall back to uppercasing the slug — handles future services
            // not yet documented (e.g. "new-service" -> "NEW_SERVICE")
            let config_name = entry
                .enable
                .as_ref()
                .and_then(|e| e.platys_init.clone())
                .unwrap_or_else(|| slug.to_uppercase().replace('-', "_"));

            slug_to_config.insert(slug.clone(), config_name.clone());

            if let Some(display_name) = &entry.name {
                services.insert(
                    config_name.clone(),
                    ServiceMeta {
                        name: display_name.clone(),
                        description: entry.description.clone().unwrap_or_default(),
                    },
                );
            }

            if let Some(props) = entry.properties {
                for (prop_key, prop_entry) in props {
                    properties.insert(prop_key, prop_entry);
                }
            }
        }

        //Build category map from index.yml
        let mut categories: HashMap<String, String> = HashMap::new();


        for section in &index_file.configuration {
            //skip the top-level section as they do not pertain to services
            let Some(subsections) = &section.sections else {
                continue;
            };
            for category in subsections {
                let Some(slugs) = &category.services else {
                    continue;
                };
                for slug in slugs {
                    //Map the slug to the config name we recorded earlier
                    let config_name = slug_to_config
                        .get(slug)
                        .cloned()
                        //slug is not know try as best to convert it
                        .unwrap_or_else(|| slug.to_uppercase().replace('-', "_"));

                    categories.insert(config_name, category.name.clone());
                }
            }
        }
        Ok(Self {
            services,
            properties,
            categories,
        })
    }
    pub fn empty() -> Self {
        Self {
            services: HashMap::new(),
            properties: HashMap::new(),
            categories: HashMap::new(),
        }
    }
}

pub fn read_local(folder: &str) -> anyhow::Result<PlatysIndex> {
    use anyhow::Context;
    let services_raw = std::fs::read_to_string(format!("{folder}/services.yml"))
        .with_context(|| format!("Cannot read {folder}/services.yml"))?;
    let index_raw = std::fs::read_to_string(format!("{folder}/index.yml"))
        .with_context(|| format!("Cannot read {folder}/index.yml"))?;

    PlatysIndex::build(services_raw.as_str(), index_raw.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_index() -> PlatysIndex {
        let folder = concat!(env!("CARGO_MANIFEST_DIR"), "/sample_files");
        read_local(folder).expect("should build index from sample_files")
    }

    #[test]
    fn resolves_display_name_and_category() {
        let idx = sample_index();

        let akhq = idx.services.get("AKHQ").expect("AKHQ should be present");
        assert_eq!(akhq.name, "AKHQ");
        assert!(!akhq.description.is_empty());

        assert_eq!(
            idx.categories.get("AKHQ").map(String::as_str),
            Some("Stream Processing"),
        );
    }

    #[test]
    fn collects_property_metadata() {
        let idx = sample_index();

        let prop = idx
            .properties
            .get("AKHQ_topic_page_size")
            .expect("AKHQ_topic_page_size should be present");
        assert!(prop.description.is_some());
        assert!(prop.default.is_some());
    }
}

