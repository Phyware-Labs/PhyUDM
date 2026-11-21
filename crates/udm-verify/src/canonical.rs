//! Canonical JSON and the payloads signatures and hashes are computed over.
//!
//! [`canonical_json`] is the form spec `provenance.md` defines: object keys
//! sorted by code point, compact separators, every character outside
//! printable ASCII escaped as `\uXXXX` (UTF-16 surrogate pairs above
//! U+FFFF) and floats written as Python's `repr` writes them. It is the
//! output of Python's
//! `json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=True)`.
//!
//! [`chain_canonical_json`] is the form a receiver's hash chain uses (spec
//! `evidence-package.md`, "Content hash"). It differs only in how floats
//! are written: decimal notation from exponent -5 rather than -4, and
//! exponents without zero padding (`1e-6`, `1e+16`).

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

use crate::hex;

/// Canonical JSON of `value` (spec `provenance.md`, "Signing payload").
pub fn canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, Floats::Python);
    out
}

/// Canonical JSON in the receiver hash-chain form (spec
/// `evidence-package.md`, "Content hash").
pub fn chain_canonical_json(value: &Value) -> String {
    let mut out = String::new();
    write_value(&mut out, value, Floats::Chain);
    out
}

/// A receiver's `content_hash` of a stored event: lowercase hex SHA-256 of
/// its [`chain_canonical_json`].
pub fn content_hash(event: &Value) -> String {
    hex::encode(&Sha256::digest(chain_canonical_json(event).as_bytes()))
}

/// The text a producer's event signature is computed over: the event with
/// every `null` object member (at any depth) and `provenance.signature`
/// removed, and `provenance` itself removed if that empties it.
pub fn signing_payload(event: &Value) -> String {
    let mut value = strip_nulls(event);
    if let Value::Object(map) = &mut value {
        if let Some(Value::Object(provenance)) = map.get_mut("provenance") {
            provenance.remove("signature");
            if provenance.is_empty() {
                map.remove("provenance");
            }
        }
    }
    canonical_json(&value)
}

/// The payload a batch-signed event's Merkle leaf is computed over:
/// [`signing_payload`] with `provenance.batch` removed as well.
pub fn leaf_payload(event: &Value) -> String {
    let mut value = event.clone();
    if let Some(Value::Object(provenance)) = value.get_mut("provenance") {
        provenance.remove("batch");
    }
    signing_payload(&value)
}

fn strip_nulls(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .filter(|(_, v)| !v.is_null())
                .map(|(k, v)| (k.clone(), strip_nulls(v)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(strip_nulls).collect()),
        other => other.clone(),
    }
}

#[derive(Clone, Copy)]
enum Floats {
    /// Python's `repr(float)`.
    Python,
    /// The receiver hash-chain form: as Python, but decimal from exponent
    /// -5 and exponents unpadded.
    Chain,
}

fn write_value(out: &mut String, v: &Value, floats: Floats) {
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(true) => out.push_str("true"),
        Value::Bool(false) => out.push_str("false"),
        Value::Number(n) => match n.as_f64() {
            Some(f) if n.is_f64() => out.push_str(&match floats {
                Floats::Python => python_float_repr(f),
                Floats::Chain => chain_float_repr(f),
            }),
            _ => out.push_str(&n.to_string()),
        },
        Value::String(s) => write_string(out, s),
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_value(out, item, floats);
            }
            out.push(']');
        }
        Value::Object(map) => write_object(out, map, floats),
    }
}

fn write_object(out: &mut String, map: &Map<String, Value>, floats: Floats) {
    let mut entries: Vec<(&String, &Value)> = map.iter().collect();
    // Code-point order. Rust compares `str` by UTF-8 bytes, which orders
    // code points the same way.
    entries.sort_by(|a, b| a.0.cmp(b.0));
    out.push('{');
    for (i, (k, v)) in entries.into_iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        write_string(out, k);
        out.push(':');
        write_value(out, v, floats);
    }
    out.push('}');
}

fn write_string(out: &mut String, s: &str) {
    use std::fmt::Write as _;
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ' '..='~' => out.push(c),
            c => {
                let mut buf = [0u16; 2];
                for unit in c.encode_utf16(&mut buf).iter() {
                    // Writing to a String cannot fail.
                    let _ = write!(out, "\\u{unit:04x}");
                }
            }
        }
    }
    out.push('"');
}

/// Format a finite `f64` exactly like Python's `repr(float)`: decimal
/// notation for decimal exponents -4 to 15, otherwise `d.ddde±XX` with at
/// least two exponent digits.
fn python_float_repr(f: f64) -> String {
    float_repr(f, -4, 2)
}

/// Format a finite `f64` in the hash-chain form: decimal notation for
/// decimal exponents -5 to 15, otherwise `d.ddde±X` without padding.
fn chain_float_repr(f: f64) -> String {
    float_repr(f, -5, 1)
}

/// Shortest round-trip digits of `f` (ties to even, as Python), written in
/// decimal notation when `min_exp <= exp < 16` and in scientific notation
/// with at least `exp_width` exponent digits otherwise.
fn float_repr(f: f64, min_exp: i32, exp_width: usize) -> String {
    if f == 0.0 {
        return if f.is_sign_negative() { "-0.0" } else { "0.0" }.to_string();
    }
    // The first correctly rounded precision that round-trips. Rust's `{:e}`
    // is also shortest, but breaks exact ties upward where Python rounds
    // half to even (e.g. float32 values widened to f64:
    // 0.0215625762939453125 is `0.021562576293945312` in Python).
    let sci = (0..17)
        .map(|p| format!("{:.*e}", p, f.abs()))
        .find(|s| s.parse::<f64>().ok() == Some(f.abs()))
        .unwrap_or_else(|| format!("{:.16e}", f.abs()));
    let (mantissa, exp) = sci.split_once('e').unwrap_or((sci.as_str(), "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();

    let body = if (min_exp..16).contains(&exp) {
        let point = exp + 1;
        match usize::try_from(point) {
            Err(_) | Ok(0) => {
                let zeros = usize::try_from(point.unsigned_abs()).unwrap_or(0);
                format!("0.{}{digits}", "0".repeat(zeros))
            }
            Ok(p) if p >= digits.len() => {
                format!("{digits}{}.0", "0".repeat(p - digits.len()))
            }
            Ok(p) => {
                let (int, frac) = digits.split_at(p);
                format!("{int}.{frac}")
            }
        }
    } else {
        let (first, rest) = digits.split_at(1.min(digits.len()));
        let mantissa = if rest.is_empty() {
            first.to_string()
        } else {
            format!("{first}.{rest}")
        };
        let sign = if exp < 0 { '-' } else { '+' };
        format!(
            "{mantissa}e{sign}{:0width$}",
            exp.unsigned_abs(),
            width = exp_width
        )
    };
    if f.is_sign_negative() {
        format!("-{body}")
    } else {
        body
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn floats_match_python_repr() {
        for (f, expected) in [
            (0.87, "0.87"),
            (100.0, "100.0"),
            (-0.021_562_576_293_945_312, "-0.021562576293945312"),
            (1e-05, "1e-05"),
            (1e15, "1000000000000000.0"),
            (1e16, "1e+16"),
            (-0.0, "-0.0"),
        ] {
            assert_eq!(python_float_repr(f), expected);
        }
    }

    #[test]
    fn chain_floats() {
        for (f, expected) in [
            (1e16, "1e+16"),
            (1e15, "1000000000000000.0"),
            (1e-5, "0.00001"),
            (1.5e-5, "0.000015"),
            (1e-6, "1e-6"),
            (2.5e-7, "2.5e-7"),
            (1.5e300, "1.5e+300"),
            (5e-324, "5e-324"),
            (100.0, "100.0"),
            (-0.0, "-0.0"),
        ] {
            assert_eq!(chain_float_repr(f), expected);
        }
    }

    #[test]
    fn chain_form_differs_only_in_float_notation() {
        let v = json!({"b": 1e-5, "c": 1e16, "a": "é\u{1F600}", "n": null, "i": 3});
        assert_eq!(
            canonical_json(&v),
            r#"{"a":"\u00e9\ud83d\ude00","b":1e-05,"c":1e+16,"i":3,"n":null}"#
        );
        assert_eq!(
            chain_canonical_json(&v),
            r#"{"a":"\u00e9\ud83d\ude00","b":0.00001,"c":1e+16,"i":3,"n":null}"#
        );
    }

    #[test]
    fn signing_payload_drops_nulls_signature_and_empty_provenance() {
        let v = json!({"x": null, "y": [null, {"z": null}], "provenance": {"signature": "ab"}});
        assert_eq!(signing_payload(&v), r#"{"y":[null,{}]}"#);
    }
}
