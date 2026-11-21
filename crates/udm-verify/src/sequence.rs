//! Ordering and replay (spec `provenance.md`, "Ordering and replay").
//!
//! [`SequenceTracker`] follows each source's highest sequence number, as a
//! receiver does, and classifies every event:
//!
//! | Status        | Meaning                                                              |
//! |---------------|----------------------------------------------------------------------|
//! | `first`       | First sequenced event of the source                                  |
//! | `in_order`    | Exactly the next number                                              |
//! | `gap`         | Ahead of the next number; the skipped range is recorded as a gap     |
//! | `restart`     | Ahead, from a new boot epoch (a producer restart)                    |
//! | `late`        | Behind, but fills a recorded gap (e.g. sent from an offline buffer)  |
//! | `duplicate`   | Behind and not in a gap: that number was already seen (a replay)     |
//! | `reset`       | Behind, from a new boot epoch: the producer's counter went back      |
//! | `regression`  | The hardware boot counter went backwards                             |
//! | `unsequenced` | No sequence number                                                   |
//! | `untrusted`   | Signature not verified, so not used for tracking                     |
//!
//! Only trusted events move the high-water mark, so forged events cannot
//! push it forward and make genuine ones look like replays.

use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;
use serde_json::Value;

/// Sequence classification of one event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SequenceStatus {
    /// First sequenced event of the source.
    First,
    /// Exactly the next number.
    InOrder,
    /// Ahead of the next number.
    Gap,
    /// Ahead, from a new boot epoch.
    Restart,
    /// Behind, filling a recorded gap.
    Late,
    /// Behind and already seen.
    Duplicate,
    /// Behind, from a new boot epoch.
    Reset,
    /// The boot counter went backwards.
    Regression,
    /// No sequence number.
    Unsequenced,
    /// Signature not verified.
    Untrusted,
}

impl SequenceStatus {
    /// The status as a string.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::First => "first",
            Self::InOrder => "in_order",
            Self::Gap => "gap",
            Self::Restart => "restart",
            Self::Late => "late",
            Self::Duplicate => "duplicate",
            Self::Reset => "reset",
            Self::Regression => "regression",
            Self::Unsequenced => "unsequenced",
            Self::Untrusted => "untrusted",
        }
    }

    /// Whether the status points at replay or rollback (`duplicate`,
    /// `reset` or `regression`).
    pub fn is_suspicious(self) -> bool {
        matches!(self, Self::Duplicate | Self::Reset | Self::Regression)
    }
}

/// The sequence fields of an event.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SequenceInfo {
    /// `provenance.sequence_num` (or the envelope's `sequence_num`).
    pub sequence: Option<i64>,
    /// `provenance.boot_epoch`.
    pub boot_epoch: Option<String>,
    /// `provenance.boot_counter`.
    pub boot_counter: Option<i64>,
}

impl SequenceInfo {
    /// Read from the signed provenance block, falling back to the
    /// envelope's `sequence_num` for older producers.
    pub fn from_event(event: &Value) -> Self {
        let provenance = event.get("provenance");
        let int = |v: Option<&Value>| v.and_then(Value::as_i64);
        Self {
            sequence: int(provenance.and_then(|p| p.get("sequence_num")))
                .or_else(|| int(event.get("sequence_num"))),
            boot_epoch: provenance
                .and_then(|p| p.get("boot_epoch"))
                .and_then(Value::as_str)
                .map(str::to_string),
            boot_counter: int(provenance.and_then(|p| p.get("boot_counter"))),
        }
    }
}

/// A range of sequence numbers a source skipped and has not filled.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub struct Gap {
    /// First missing number.
    pub first_missing: i64,
    /// Last missing number.
    pub last_missing: i64,
}

#[derive(Debug, Default)]
struct SourceState {
    high_water: i64,
    boot_epoch: Option<String>,
    boot_counter: Option<i64>,
    epochs: BTreeSet<String>,
    gaps: BTreeSet<Gap>,
}

/// Per-source sequence tracking.
#[derive(Debug, Default)]
pub struct SequenceTracker {
    sources: BTreeMap<String, SourceState>,
}

impl SequenceTracker {
    /// An empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Classify the next event received from `source_id`, and update the
    /// source's state if `trusted` (its signature verified).
    pub fn observe(
        &mut self,
        source_id: &str,
        info: &SequenceInfo,
        trusted: bool,
    ) -> SequenceStatus {
        let Some(seq) = info.sequence else {
            return SequenceStatus::Unsequenced;
        };
        if !trusted {
            return SequenceStatus::Untrusted;
        }
        let Some(state) = self.sources.get_mut(source_id) else {
            let mut state = SourceState {
                high_water: seq,
                boot_epoch: info.boot_epoch.clone(),
                boot_counter: info.boot_counter,
                ..SourceState::default()
            };
            state.epochs.extend(info.boot_epoch.clone());
            self.sources.insert(source_id.to_string(), state);
            return SequenceStatus::First;
        };
        let status = classify(state, seq, info);
        if let Some(epoch) = &info.boot_epoch {
            if !matches!(
                status,
                SequenceStatus::Duplicate | SequenceStatus::Regression
            ) {
                state.epochs.insert(epoch.clone());
            }
        }
        status
    }

    /// The gaps still open, by source.
    pub fn open_gaps(&self) -> BTreeMap<&str, Vec<Gap>> {
        self.sources
            .iter()
            .filter(|(_, s)| !s.gaps.is_empty())
            .map(|(id, s)| (id.as_str(), s.gaps.iter().copied().collect()))
            .collect()
    }
}

fn classify(state: &mut SourceState, seq: i64, info: &SequenceInfo) -> SequenceStatus {
    let counter_went_back = matches!(
        (info.boot_counter, state.boot_counter),
        (Some(new), Some(old)) if new < old
    );
    let new_epoch = info.boot_epoch.is_some() && info.boot_epoch != state.boot_epoch;
    if seq > state.high_water {
        if counter_went_back {
            return SequenceStatus::Regression;
        }
        let gap = (seq > state.high_water + 1).then(|| Gap {
            first_missing: state.high_water + 1,
            last_missing: seq - 1,
        });
        let status = if new_epoch {
            SequenceStatus::Restart
        } else if gap.is_some() {
            SequenceStatus::Gap
        } else {
            SequenceStatus::InOrder
        };
        state.gaps.extend(gap);
        advance(state, seq, info);
        return status;
    }
    let containing = state
        .gaps
        .iter()
        .find(|g| (g.first_missing..=g.last_missing).contains(&seq))
        .copied();
    if let Some(gap) = containing {
        state.gaps.remove(&gap);
        if gap.first_missing < seq {
            state.gaps.insert(Gap {
                first_missing: gap.first_missing,
                last_missing: seq - 1,
            });
        }
        if seq < gap.last_missing {
            state.gaps.insert(Gap {
                first_missing: seq + 1,
                last_missing: gap.last_missing,
            });
        }
        return SequenceStatus::Late;
    }
    let epoch_seen = info
        .boot_epoch
        .as_ref()
        .is_some_and(|e| state.epochs.contains(e));
    if new_epoch && !epoch_seen {
        if counter_went_back {
            return SequenceStatus::Regression;
        }
        // A new run that started below the high-water mark: the producer's
        // sequence state was lost or rolled back. Follow the new run.
        advance(state, seq, info);
        return SequenceStatus::Reset;
    }
    SequenceStatus::Duplicate
}

fn advance(state: &mut SourceState, seq: i64, info: &SequenceInfo) {
    state.high_water = seq;
    if info.boot_epoch.is_some() {
        state.boot_epoch.clone_from(&info.boot_epoch);
    }
    state.boot_counter = match (info.boot_counter, state.boot_counter) {
        (Some(new), Some(old)) => Some(new.max(old)),
        (new, old) => new.or(old),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info(seq: i64, epoch: &str, counter: Option<i64>) -> SequenceInfo {
        SequenceInfo {
            sequence: Some(seq),
            boot_epoch: Some(epoch.to_string()),
            boot_counter: counter,
        }
    }

    #[test]
    fn classifies_a_stream() {
        let mut t = SequenceTracker::new();
        let mut see = |seq, epoch, counter| t.observe("s", &info(seq, epoch, counter), true);
        assert_eq!(see(1, "a", Some(1)), SequenceStatus::First);
        assert_eq!(see(2, "a", Some(1)), SequenceStatus::InOrder);
        assert_eq!(see(5, "a", Some(1)), SequenceStatus::Gap);
        assert_eq!(see(3, "a", Some(1)), SequenceStatus::Late);
        assert_eq!(see(3, "a", Some(1)), SequenceStatus::Duplicate);
        assert_eq!(see(6, "b", Some(2)), SequenceStatus::Restart);
        assert_eq!(see(7, "c", Some(1)), SequenceStatus::Regression);
        assert_eq!(see(2, "d", Some(3)), SequenceStatus::Reset);
        assert_eq!(
            t.open_gaps().get("s"),
            Some(&vec![Gap {
                first_missing: 4,
                last_missing: 4
            }])
        );
    }

    #[test]
    fn untrusted_events_do_not_move_the_mark() {
        let mut t = SequenceTracker::new();
        assert_eq!(
            t.observe("s", &info(100, "a", None), false),
            SequenceStatus::Untrusted
        );
        assert_eq!(
            t.observe("s", &info(1, "a", None), true),
            SequenceStatus::First
        );
        assert_eq!(
            t.observe("s", &SequenceInfo::default(), true),
            SequenceStatus::Unsequenced
        );
    }
}
