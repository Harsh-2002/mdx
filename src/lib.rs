pub mod cli;
pub mod completions;
pub mod diff;
pub mod export;
#[cfg(feature = "url")]
pub mod fetch;
pub mod fmt;
pub mod html;
pub mod lint;
pub mod parse;
#[cfg(feature = "watch")]
pub mod present;
pub mod publish;
pub mod render;
#[cfg(feature = "serve")]
pub mod serve;
pub mod stats;
pub mod style;
pub mod terminal;
pub mod text;
pub mod toc;
#[cfg(feature = "url")]
pub mod update;
#[cfg(feature = "watch")]
pub mod watch;

/// Estimate LLM token count. Heuristic: ~4 characters per token.
pub fn estimate_tokens(text: &str) -> u64 {
    (text.len() as f64 / 4.0).ceil() as u64
}
