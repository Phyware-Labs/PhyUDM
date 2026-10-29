//! Client-side aggregation over a fetched event set.
//!
//! PhyCloud has no aggregate endpoint, so the adapter computes
//! `agg_fn(field) GROUP BY group_by[]` over the events it fetched (already
//! filtered by predicate + time range). Buckets are keyed by the group-by
//! field values and emitted in deterministic key order.

use std::collections::BTreeMap;

use serde_json::Value;
use udm_eventstore::{
    AggregateBucket, AggregateFn, AggregateQuery, AggregateResult, Error, Result, UdmEvent,
};

/// JSON-Pointer-style field lookup rooted at the event envelope.
fn point<'a>(root: &'a Value, field: &str) -> Option<&'a Value> {
    let trimmed = field.trim_start_matches('/');
    if trimmed.is_empty() {
        return Some(root);
    }
    root.pointer(&format!("/{trimmed}"))
}

fn as_f64(v: &Value) -> Option<f64> {
    v.as_f64()
}

/// Accumulator for one group bucket.
#[derive(Default)]
struct Acc {
    key: BTreeMap<String, Value>,
    count: u64,
    sum: f64,
    min: Option<f64>,
    max: Option<f64>,
    numeric_count: u64,
}

impl Acc {
    fn observe_value(&mut self, x: f64) {
        self.sum += x;
        self.numeric_count += 1;
        self.min = Some(self.min.map_or(x, |m| m.min(x)));
        self.max = Some(self.max.map_or(x, |m| m.max(x)));
    }

    fn finish(self, agg_fn: AggregateFn) -> Option<AggregateBucket> {
        let value = match agg_fn {
            AggregateFn::Count => self.count as f64,
            AggregateFn::Sum => self.sum,
            AggregateFn::Avg => {
                if self.numeric_count == 0 {
                    return None;
                }
                self.sum / self.numeric_count as f64
            }
            AggregateFn::Min => self.min?,
            AggregateFn::Max => self.max?,
        };
        Some(AggregateBucket {
            key: self.key,
            value,
            event_count: self.count,
        })
    }
}

/// Compute the aggregate over `events`.
pub fn compute_aggregate(agg: &AggregateQuery, events: &[UdmEvent]) -> Result<AggregateResult> {
    if !matches!(agg.agg_fn, AggregateFn::Count) && agg.field.is_none() {
        return Err(Error::InvalidQuery(format!(
            "aggregate {:?} requires a `field`",
            agg.agg_fn
        )));
    }

    // Deterministic bucket ordering keyed by the stringified group key.
    let mut buckets: BTreeMap<String, Acc> = BTreeMap::new();

    for event in events {
        let root = serde_json::to_value(event)?;

        // Build the group key from the group_by field paths.
        let mut key = BTreeMap::new();
        for path in &agg.group_by {
            let val = point(&root, path).cloned().unwrap_or(Value::Null);
            key.insert(path.clone(), val);
        }
        let key_str = serde_json::to_string(&key).unwrap_or_default();

        let acc = buckets.entry(key_str).or_default();
        if acc.count == 0 {
            acc.key = key;
        }
        acc.count += 1;

        if let Some(field) = &agg.field {
            if let Some(x) = point(&root, field).and_then(as_f64) {
                acc.observe_value(x);
            }
        }
    }

    let out: Vec<AggregateBucket> = buckets
        .into_values()
        .filter_map(|acc| acc.finish(agg.agg_fn))
        .collect();

    Ok(AggregateResult {
        buckets: out,
        agg_fn: agg.agg_fn,
    })
}
