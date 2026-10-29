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
pub const DEFAULT_VERSION: &str = "0.0.3";

/// All schema versions the CLI knows about.
pub const KNOWN_VERSIONS: &[&str] = &["0.0.3"];

/// Comma-separated list of [`KNOWN_VERSIONS`] (used in error messages).
pub fn supported_versions_csv() -> String {
    KNOWN_VERSIONS.join(", ")
}

// ---- v0.0.3 embedded artifacts -----------------------------------------

const V003_EVENT: &str = include_str!("../../../schemas/v0.0.3/event.schema.json");
const V003_ENVELOPE: &str = include_str!("../../../schemas/v0.0.3/envelope.schema.json");
const V003_OBJECT_REF: &str = include_str!("../../../schemas/v0.0.3/object_ref.schema.json");

macro_rules! embed_domain {
    ($num:expr, $slug:literal) => {
        (
            $slug,
            include_str!(concat!(
                "../../../schemas/v0.0.3/domains/",
                stringify!($num),
                "-",
                $slug,
                ".schema.json"
            )),
        )
    };
}

// NB: stringify!(02) → "02" so the macro produces the right filename.
const V003_DOMAINS: &[(&str, &str)] = &[
    embed_domain!(01, "identity"),
    embed_domain!(02, "location"),
    embed_domain!(03, "motion"),
    embed_domain!(04, "power"),
    embed_domain!(05, "operational"),
    embed_domain!(06, "navigation"),
    embed_domain!(07, "perception"),
    embed_domain!(08, "safety"),
    embed_domain!(09, "actuators"),
    embed_domain!(10, "communication"),
    embed_domain!(11, "compute"),
    embed_domain!(12, "ai"),
    embed_domain!(13, "maintenance"),
    embed_domain!(14, "context"),
    embed_domain!(15, "payload"),
    embed_domain!(16, "manipulation"),
    embed_domain!(17, "hri"),
    embed_domain!(18, "coordination"),
    embed_domain!(19, "simulation"),
    embed_domain!(20, "thermal"),
    embed_domain!(21, "audio"),
    embed_domain!(22, "environment-interaction"),
    embed_domain!(23, "compliance"),
];

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
        "0.0.3" => Ok(load_v003()),
        other => Err(CliError::UnknownSchemaVersion(other.to_owned())),
    }
}

fn load_v003() -> SchemaBundle {
    let mut domains = BTreeMap::new();
    for (slug, body) in V003_DOMAINS {
        // schema file slug → canonical key:
        //   "identity"                 -> "identity"
        //   "environment-interaction"  -> "environment_interaction"
        let key = (*slug).replace('-', "_");
        let value: Value = serde_json::from_str(body)
            .unwrap_or_else(|err| panic!("invalid embedded domain {slug}: {err}"));
        domains.insert(key, value);
    }
    SchemaBundle {
        version: "0.0.3".to_owned(),
        event: serde_json::from_str(V003_EVENT)
            .unwrap_or_else(|err| panic!("invalid embedded event schema: {err}")),
        envelope: serde_json::from_str(V003_ENVELOPE)
            .unwrap_or_else(|err| panic!("invalid embedded envelope schema: {err}")),
        object_ref: serde_json::from_str(V003_OBJECT_REF)
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
    fn unknown_version_errors() {
        let err = load_version("9.9.9").unwrap_err();
        assert!(matches!(err, CliError::UnknownSchemaVersion(_)));
    }
}
