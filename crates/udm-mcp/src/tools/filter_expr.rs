//! Filter expression parser shared by `query_events` + `aggregate`.

use udm_eventstore::Predicate;

pub fn parse_filters(exprs: &[String]) -> Result<Predicate, String> {
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

fn parse_one(expr: &str) -> Result<Predicate, String> {
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
        return Ok(Predicate::In {
            field: field.trim().to_owned(),
            values: parse_in_list(rest.trim()),
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
    Err(format!(
        "unrecognised filter expression {expr:?}; \
         expected `field=value`, `field!=value`, `field in [a,b]`, `field contains text`, or `field exists`"
    ))
}

fn split_keyword<'a>(haystack: &'a str, needle: &str) -> Option<(&'a str, &'a str)> {
    let lower = haystack.to_ascii_lowercase();
    let pos = lower.find(needle)?;
    Some((&haystack[..pos], &haystack[pos + needle.len()..]))
}

fn parse_value(raw: &str) -> serde_json::Value {
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
