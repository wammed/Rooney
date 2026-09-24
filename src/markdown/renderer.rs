//! Backward-compatibility bridge for `rooney::markdown::renderer`.
//! Subsystems have been organized into:
//! - `html`: Safe text-oriented HTML fallback parsing
//! - `inline`: Rich InlineSpan, InlineStyle, autolinks, and InlineCollector
//! - `parser`: MarkdownBlock AST, table blocks, list context, and pulldown-cmark parser

pub use super::html::*;
pub use super::inline::*;
pub use super::parser::*;
