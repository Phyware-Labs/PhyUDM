//! Schemas embedded into the `udm` binary so it is self-contained.
//!
//! The CLI ships every JSON Schema artifact under
//! `schemas/v<version>/` at build time via [`include_str!`]. Adding a
//! new schema version means adding it to [`KNOWN_VERSIONS`] and to the
//! [`load_version`] dispatcher.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{CliError, CliResult};

/// Default schema version when none is specified on the command line.
pub const DEFAULT_VERSION: &str = "0.0.5";

/// All schema versions the CLI knows about.
pub const KNOWN_VERSIONS: &[&str] = &["0.0.3", "0.0.4", "0.0.5"];

/// Comma-separated list of [`KNOWN_VERSIONS`] (used in error messages).
pub fn supported_versions_csv() -> String {
    KNOWN_VERSIONS.join(", ")
}

/// Every artifact of one schema version, as embedded text.
struct Embedded {
    version: &'static str,
    event: &'static str,
    envelope: &'static str,
    object_ref: &'static str,
    domains: &'static [(&'static str, &'static str)],
}

macro_rules! embed_domain {
    ($dir:literal, $num:expr, $slug:literal) => {
        (
            $slug,
            include_str!(concat!(
                "../../../schemas/",
                $dir,
                "/domains/",
                stringify!($num),
                "-",
                $slug,
                ".schema.json"
            )),
        )
    };
}

/// Embed every artifact under `schemas/<dir>/`.
macro_rules! embed_version {
    ($version:literal, $dir:literal) => {
        Embedded {
            version: $version,
            event: include_str!(concat!("../../../schemas/", $dir, "/event.schema.json")),
            envelope: include_str!(concat!("../../../schemas/", $dir, "/envelope.schema.json")),
            object_ref: include_str!(concat!(
                "../../../schemas/",
                $dir,
                "/object_ref.schema.json"
            )),
            // NB: stringify!(02) → "02" so the macro produces the right filename.
            domains: &[
                embed_domain!($dir, 01, "identity"),
                embed_domain!($dir, 02, "location"),
                embed_domain!($dir, 03, "motion"),
                embed_domain!($dir, 04, "power"),
                embed_domain!($dir, 05, "operational"),
                embed_domain!($dir, 06, "navigation"),
                embed_domain!($dir, 07, "perception"),
                embed_domain!($dir, 08, "safety"),
                embed_domain!($dir, 09, "actuators"),
                embed_domain!($dir, 10, "communication"),
                embed_domain!($dir, 11, "compute"),
                embed_domain!($dir, 12, "ai"),
                embed_domain!($dir, 13, "maintenance"),
                embed_domain!($dir, 14, "context"),
                embed_domain!($dir, 15, "payload"),
                embed_domain!($dir, 16, "manipulation"),
                embed_domain!($dir, 17, "hri"),
                embed_domain!($dir, 18, "coordination"),
                embed_domain!($dir, 19, "simulation"),
                embed_domain!($dir, 20, "thermal"),
                embed_domain!($dir, 21, "audio"),
                embed_domain!($dir, 22, "environment-interaction"),
                embed_domain!($dir, 23, "compliance"),
            ],
        }
    };
}

const V003: Embedded = embed_version!("0.0.3", "v0.0.3");
const V004: Embedded = embed_version!("0.0.4", "v0.0.4");
const V005: Embedded = embed_version!("0.0.5", "v0.0.5");

/// Bundle of every artifact for one schema version, materialised as
/// `serde_json::Value` for further processing.
pub struct SchemaBundle {
    /// Schema version (e.g. `"0.0.3"`).
    #[allow(dead_code)]
    pub version: String,
    /// The `event.schema.json` artifact.
    pub event: Value,
    /// The `envelope.schema.json` artifact.
    pub envelope: Value,
    /// The `object_ref.schema.json` artifact.
    pub object_ref: Value,
    /// Domain artifacts keyed by canonical domain key (e.g.
    /// `"identity"`, `"environment_interaction"`). Slugs in the
    /// schema filenames use dashes; the map normalises to the
    /// `event` schema's underscore form.
    pub domains: BTreeMap<String, Value>,
}

impl std::fmt::Debug for SchemaBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SchemaBundle")
            .field("version", &self.version)
            .field("domains_loaded", &self.domains.len())
            .finish_non_exhaustive()
    }
}

impl SchemaBundle {
    /// Look up an artifact by name. `"event"` / `"envelope"` /
    /// `"object_ref"` return the corresponding top-level schema; any
    /// other key is treated as a domain key.
    pub fn artifact(&self, name: &str) -> Option<&Value> {
        match name {
            "event" => Some(&self.event),
            "envelope" => Some(&self.envelope),
            "object_ref" => Some(&self.object_ref),
            other => self.domains.get(other).or_else(|| {
                // Accept the dashed filename form too.
                self.domains.get(&other.replace('-', "_"))
            }),
        }
    }

    /// Iterate every artifact as `(name, schema)` pairs. Useful for
    /// validators that need to register every schema up front.
    pub fn iter_artifacts(&self) -> impl Iterator<Item = (String, &Value)> {
        let head = [
            ("event".to_owned(), &self.event),
            ("envelope".to_owned(), &self.envelope),
            ("object_ref".to_owned(), &self.object_ref),
        ];
        let domains = self.domains.iter().map(|(k, v)| (k.clone(), v));
        head.into_iter().chain(domains)
    }
}

/// Materialise the schema bundle for `version`.
pub fn load_version(version: &str) -> CliResult<SchemaBundle> {
    match version {
        "0.0.3" => Ok(load(&V003)),
        "0.0.4" => Ok(load(&V004)),
        "0.0.5" => Ok(load(&V005)),
        other => Err(CliError::UnknownSchemaVersion(other.to_owned())),
    }
}

fn load(embedded: &Embedded) -> SchemaBundle {
    let mut domains = BTreeMap::new();
    for (slug, body) in embedded.domains {
        // schema file slug → canonical key:
        //   "identity"                 -> "identity"
        //   "environment-interaction"  -> "environment_interaction"
        let key = (*slug).replace('-', "_");
        let value: Value = serde_json::from_str(body)
            .unwrap_or_else(|err| panic!("invalid embedded domain {slug}: {err}"));
        domains.insert(key, value);
    }
    SchemaBundle {
        version: embedded.version.to_owned(),
        event: serde_json::from_str(embedded.event)
            .unwrap_or_else(|err| panic!("invalid embedded event schema: {err}")),
        envelope: serde_json::from_str(embedded.envelope)
            .unwrap_or_else(|err| panic!("invalid embedded envelope schema: {err}")),
        object_ref: serde_json::from_str(embedded.object_ref)
            .unwrap_or_else(|err| panic!("invalid embedded object_ref schema: {err}")),
        domains,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn v003_loads() {
        let bundle = load_version("0.0.3").unwrap();
        assert_eq!(bundle.domains.len(), 23);
        assert!(bundle.domains.contains_key("environment_interaction"));
        assert!(bundle.artifact("event").is_some());
        assert!(bundle.artifact("safety").is_some());
        // Canonical and dashed lookup both work.
        assert!(bundle.artifact("environment-interaction").is_some());
    }

    #[test]
    fn v004_adds_provenance_and_heartbeat() {
        let bundle = load_version("0.0.4").unwrap();
        assert_eq!(bundle.version, "0.0.4");
        assert_eq!(bundle.domains.len(), 23);
        let props = &bundle.event["properties"];
        assert!(props["heartbeat"].is_object());
        assert!(props["provenance"]["properties"]["batch"].is_object());
        // v0.0.3 stays as published.
        let v003 = load_version("0.0.3").unwrap();
        assert!(v003.event["properties"].get("heartbeat").is_none());
        let v005 = load_version("0.0.5").unwrap();
        assert!(v005.event["properties"]["ros_graph"].is_object());
        assert!(bundle.event["properties"].get("ros_graph").is_none());
        assert_eq!(DEFAULT_VERSION, "0.0.5");
    }

    #[test]
    fn unknown_version_errors() {
        let err = load_version("9.9.9").unwrap_err();
        assert!(matches!(err, CliError::UnknownSchemaVersion(_)));
    }
}
