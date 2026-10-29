//! Predicate parser shared by `udm query` and `udm aggregate`.
//!
//! Supported expressions:
//!   - `field=value`           → Eq (value parsed as JSON; quoted strings work)
//!   - `field!=value`          → Ne
//!   - `field in [a,b,c]`      → In
//!   - `field contains text`   → Contains
//!   - `field exists`          → Exists

use udm_eventstore::Predicate;

use crate::{CliError, CliResult};

/// Parse a list of `--filter` expressions into a single `And`
/// predicate. Empty list yields `Predicate::And(vec![])`.
pub fn parse(exprs: &[String]) -> CliResult<Predicate> {
    let mut parts = Vec::with_capacity(exprs.len());
    for e in exprs {
        parts.push(parse_one(e.trim())?);
    }
    if parts.len() == 1 {
        Ok(parts.into_iter().next().expect("len == 1"))
    } else {
        Ok(Predicate::And(parts))
    }
}

fn parse_one(expr: &str) -> CliResult<Predicate> {
    // Try most-specific operators first.
    if let Some((field, raw)) = expr.split_once("!=") {
        return Ok(Predicate::Ne {
            field: field.trim().to_owned(),
            value: parse_value(raw.trim()),
        });
    }
    if let Some((field, rest)) = split_keyword(expr, " contains ") {
        return Ok(Predicate::Contains {
            field: field.trim().to_owned(),
            value: strip_quotes(rest.trim()).to_owned(),
        });
    }
    if let Some((field, rest)) = split_keyword(expr, " in ") {
        let list = parse_in_list(rest.trim());
        return Ok(Predicate::In {
            field: field.trim().to_owned(),
            values: list,
        });
    }
    if let Some(field) = expr.strip_suffix(" exists") {
        return Ok(Predicate::Exists {
            field: field.trim().to_owned(),
        });
    }
    if let Some((field, raw)) = expr.split_once('=') {
        return Ok(Predicate::Eq {
            field: field.trim().to_owned(),
            value: parse_value(raw.trim()),
        });
    }
    Err(CliError::Usage(format!(
        "unrecognised --filter expression {expr:?}; \
         expected `field=value`, `field!=value`, `field in [a,b]`, `field contains text`, or `field exists`"
    )))
}

fn split_keyword<'a>(haystack: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let lower = haystack.to_ascii_lowercase();
    let pos = lower.find(needle)?;
    Some((&haystack[..pos], &haystack[pos + needle.len()..]))
}

fn parse_value(raw: &str) -> serde_json::Value {
    // Try JSON first; fall back to a string.
    serde_json::from_str(raw).unwrap_or_else(|_| {
        let stripped = strip_quotes(raw);
        serde_json::Value::String(stripped.to_owned())
    })
}

fn parse_in_list(raw: &str) -> Vec<serde_json::Value> {
    let trimmed = raw.trim().trim_start_matches('[').trim_end_matches(']');
    trimmed
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(parse_value)
        .collect()
}

fn strip_quotes(s: &str) -> &str {
    s.trim_matches('"').trim_matches('\'')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_eq_with_string_value() {
        let p = parse_one("event_type=safety_violation").unwrap();
        assert!(matches!(p, Predicate::Eq { .. }));
    }

    #[test]
    fn parses_in_list() {
        let p = parse_one("source_type in [amr,agv]").unwrap();
        match p {
            Predicate::In { values, .. } => {
                assert_eq!(values.len(), 2);
            }
            other => panic!("expected In, got {other:?}"),
        }
    }

    #[test]
    fn parses_exists() {
        let p = parse_one("safety exists").unwrap();
        assert!(matches!(p, Predicate::Exists { .. }));
    }

    #[test]
    fn parses_contains() {
        let p = parse_one("event_type contains violation").unwrap();
        assert!(matches!(p, Predicate::Contains { .. }));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_one("garbage").is_err());
    }
}
