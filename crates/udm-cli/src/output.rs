//! Output formatting (JSON-Lines vs pretty).

use std::io::{self, Write};

use clap::ValueEnum;
use serde::Serialize;

/// Selectable output format for every CLI subcommand.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[clap(rename_all = "snake_case")]
pub enum OutputFormat {
    /// One JSON object per line. Default; pipes cleanly into `jq` and
    /// LLM agents.
    Jsonl,
    /// Pretty-printed JSON. Better for hand-reading.
    Pretty,
}

impl OutputFormat {
    /// Serialize a single value and write it as one line / one
    /// indented block per [`OutputFormat`] semantics.
    pub fn write_one<T: Serialize, W: Write>(self, mut out: W, value: &T) -> io::Result<()> {
        let serialized = match self {
            Self::Jsonl => serde_json::to_string(value),
            Self::Pretty => serde_json::to_string_pretty(value),
        };
        let s = serialized.map_err(io::Error::other)?;
        writeln!(out, "{s}")
    }

    /// Serialize each element of an iterable. JSON-Lines emits one
    /// line per element; Pretty emits a JSON array.
    pub fn write_iter<T, I, W>(self, mut out: W, items: I) -> io::Result<()>
    where
        T: Serialize,
        I: IntoIterator<Item = T>,
        W: Write,
    {
        match self {
            Self::Jsonl => {
                for item in items {
                    self.write_one(&mut out, &item)?;
                }
                Ok(())
            }
            Self::Pretty => {
                let collected: Vec<T> = items.into_iter().collect();
                self.write_one(&mut out, &collected)
            }
        }
    }
}
