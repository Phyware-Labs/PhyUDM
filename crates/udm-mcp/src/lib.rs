//! `udm-mcp` — the Model Context Protocol server for UDM telemetry
//! analysis.
//!
//! The server exposes the UDM analysis surface to any MCP-capable LLM
//! agent (Claude Desktop, Cursor, Copilot CLI, Continue, custom agent
//! frameworks). It is **consumer-oriented**: tools let the agent
//! reach into a UDM-conforming telemetry store and reason about what
//! an autonomous system did — they do *not* help the agent generate
//! fresh UDM payloads. That belongs to the producer SDKs and the
//! `udm template` CLI subcommand.
//!
//! Backend-agnostic by construction (consumes the `UdmEventStore`
//! trait) and ships with PhyCloud as the sole
//! reference deployment adapter at v0.0.3; the in-process `memory`
//! adapter powers the test suite and cookbook articles.

#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

pub mod server;
mod store;
mod tools;
mod util;

pub use server::UdmAnalysisServer;
pub use store::store_from_url;
