pub mod loader;
pub mod parser;
pub mod session;

pub use loader::*;
pub use parser::*;
pub use session::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LanguageDef {
    pub id: String,
    pub name: String,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnippetDef {
    pub category: String,
    pub label: String,
    pub content: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SnippetFile {
    pub language: LanguageDef,
    pub snippets: Vec<SnippetDef>,
}

