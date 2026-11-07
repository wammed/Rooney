use super::{LanguageDef, SnippetFile};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct SnippetRegistry {
    /// Maps language ID to SnippetFile
    pub languages: HashMap<String, SnippetFile>,
    /// Maps file extension (e.g. "md", "rs") to language ID
    pub ext_to_lang: HashMap<String, String>,
}

impl SnippetRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        // 1. Register default built-in markdown snippets
        registry.load_markdown_preset();

        // 2. Load from local ./snippets directory if present
        let local_snippets = Path::new("snippets");
        if local_snippets.is_dir() {
            registry.load_from_dir(local_snippets);
        }

        // 3. Load from user configuration directory (~/.config/rooney/snippets)
        if let Some(proj_dirs) = directories::ProjectDirs::from("com", "wammed", "rooney") {
            let user_snippets = proj_dirs.config_dir().join("snippets");
            if user_snippets.is_dir() {
                registry.load_from_dir(&user_snippets);
            }
        }

        registry
    }

    pub fn load_markdown_preset(&mut self) {
        let default_toml = include_str!("../../snippets/markdown.toml");
        if let Ok(file) = toml::from_str::<SnippetFile>(default_toml) {
            self.register_file(file);
        }
    }

    pub fn load_from_dir<P: AsRef<Path>>(&mut self, dir_path: P) {
        let Ok(entries) = std::fs::read_dir(dir_path) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("toml") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    if let Ok(file) = toml::from_str::<SnippetFile>(&content) {
                        self.register_file(file);
                    }
                }
            }
        }
    }

    pub fn register_file(&mut self, file: SnippetFile) {
        for ext in &file.language.extensions {
            self.ext_to_lang
                .insert(ext.to_lowercase(), file.language.id.clone());
        }
        match self.languages.entry(file.language.id.clone()) {
            std::collections::hash_map::Entry::Occupied(mut entry) => {
                let existing = entry.get_mut();
                for snip in file.snippets {
                    if let Some(pos) = existing.snippets.iter().position(|s| s.label == snip.label) {
                        existing.snippets[pos] = snip;
                    } else {
                        existing.snippets.push(snip);
                    }
                }
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(file);
            }
        }
    }

    pub fn get_language_for_ext(&self, ext: &str) -> Option<&SnippetFile> {
        let lang_id = self.ext_to_lang.get(&ext.to_lowercase())?;
        self.languages.get(lang_id)
    }

    pub fn get_language(&self, lang_id: &str) -> Option<&SnippetFile> {
        self.languages.get(lang_id)
    }

    pub fn available_languages(&self) -> Vec<&LanguageDef> {
        let mut list: Vec<&LanguageDef> = self.languages.values().map(|f| &f.language).collect();
        list.sort_by(|a, b| a.name.cmp(&b.name));
        list
    }
}
