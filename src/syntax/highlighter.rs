use crate::theme::ThemeConfig;
use cosmic::iced::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Path;
use tree_sitter::{InputEdit, Node, Parser, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    C,
    Cpp,
    Bash,
    Fish,
    Toml,
    Yaml,
    Json,
    Ini,
    Markdown,
    PlainText,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Self {
        let file_name = path
            .file_name()
            .and_then(|f| f.to_str())
            .unwrap_or("")
            .to_lowercase();

        if file_name == "cargo.lock" {
            return SupportedLanguage::Toml;
        }
        if file_name == ".gitconfig" {
            return SupportedLanguage::Ini;
        }
        if file_name == ".bashrc" || file_name == ".zshrc" || file_name == ".profile" {
            return SupportedLanguage::Bash;
        }
        if file_name == "config.fish" {
            return SupportedLanguage::Fish;
        }

        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => SupportedLanguage::Rust,
            "py" | "pyi" => SupportedLanguage::Python,
            "js" | "mjs" | "cjs" | "jsx" => SupportedLanguage::JavaScript,
            "ts" | "mts" | "cts" | "tsx" => SupportedLanguage::TypeScript,
            "c" | "h" => SupportedLanguage::C,
            "cpp" | "cc" | "cxx" | "hpp" | "hxx" => SupportedLanguage::Cpp,
            "sh" | "bash" | "zsh" => SupportedLanguage::Bash,
            "fish" => SupportedLanguage::Fish,
            "toml" => SupportedLanguage::Toml,
            "yaml" | "yml" => SupportedLanguage::Yaml,
            "json" | "jsonc" => SupportedLanguage::Json,
            "ini" | "conf" | "cfg" => SupportedLanguage::Ini,
            "md" | "markdown" => SupportedLanguage::Markdown,
            _ => SupportedLanguage::PlainText,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SupportedLanguage::Rust => "Rust",
            SupportedLanguage::Python => "Python",
            SupportedLanguage::JavaScript => "JavaScript",
            SupportedLanguage::TypeScript => "TypeScript",
            SupportedLanguage::C => "C",
            SupportedLanguage::Cpp => "C++",
            SupportedLanguage::Bash => "Bash / Shell",
            SupportedLanguage::Fish => "Fish",
            SupportedLanguage::Toml => "TOML",
            SupportedLanguage::Yaml => "YAML",
            SupportedLanguage::Json => "JSON",
            SupportedLanguage::Ini => "INI / Conf",
            SupportedLanguage::Markdown => "Markdown",
            SupportedLanguage::PlainText => "Plain Text",
        }
    }

    pub fn tree_sitter_language(&self) -> Option<tree_sitter::Language> {
        match self {
            SupportedLanguage::Rust => Some(tree_sitter_rust::LANGUAGE.into()),
            SupportedLanguage::Python => Some(tree_sitter_python::LANGUAGE.into()),
            SupportedLanguage::JavaScript => Some(tree_sitter_javascript::LANGUAGE.into()),
            SupportedLanguage::TypeScript => {
                Some(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into())
            }
            SupportedLanguage::C => Some(tree_sitter_c::LANGUAGE.into()),
            SupportedLanguage::Cpp => Some(tree_sitter_cpp::LANGUAGE.into()),
            SupportedLanguage::Bash => Some(tree_sitter_bash::LANGUAGE.into()),
            SupportedLanguage::Fish => Some(tree_sitter_fish::language()),
            SupportedLanguage::Toml => Some(tree_sitter_toml_ng::LANGUAGE.into()),
            SupportedLanguage::Yaml => Some(tree_sitter_yaml::LANGUAGE.into()),
            SupportedLanguage::Json => Some(tree_sitter_json::LANGUAGE.into()),
            SupportedLanguage::Ini | SupportedLanguage::Markdown | SupportedLanguage::PlainText => {
                None
            }
        }
    }

    pub fn line_comment_prefix(&self) -> Option<&'static str> {
        match self {
            SupportedLanguage::Rust
            | SupportedLanguage::JavaScript
            | SupportedLanguage::TypeScript
            | SupportedLanguage::C
            | SupportedLanguage::Cpp => Some("//"),
            SupportedLanguage::Python
            | SupportedLanguage::Bash
            | SupportedLanguage::Fish
            | SupportedLanguage::Toml
            | SupportedLanguage::Yaml
            | SupportedLanguage::Ini => Some("#"),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenType {
    Keyword,
    Function,
    TypeName,
    StringLit,
    NumberLit,
    Comment,
    Operator,
    Variable,
    Punctuation,
    Heading,
    Default,
}

impl TokenType {
    pub fn color(&self, theme: &ThemeConfig) -> Color {
        match self {
            TokenType::Keyword => theme.keyword,
            TokenType::Function => theme.function,
            TokenType::TypeName => theme.type_name,
            TokenType::StringLit => theme.string,
            TokenType::NumberLit => theme.number,
            TokenType::Comment => theme.comment,
            TokenType::Operator => theme.operator,
            TokenType::Variable => theme.variable,
            TokenType::Punctuation => theme.punctuation,
            TokenType::Heading => theme.accent,
            TokenType::Default => theme.fg,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    pub start_col: usize,
    pub end_col: usize,
    pub token_type: TokenType,
}

/// Helper to map UTF-8 byte offsets to 0-based character indices within a line.
///
/// Tree-sitter coordinates (`Point.column`) are byte offsets, whereas Rooney's
/// visual layout and rendering use character indices. This mapper bridges the two
/// accurately and without allocation for pure ASCII lines.
#[derive(Debug, Clone)]
pub struct ByteCharMapper<'a> {
    pub text: &'a str,
    char_byte_offsets: Option<Vec<usize>>,
    total_chars: usize,
}

impl<'a> ByteCharMapper<'a> {
    pub fn new(text: &'a str) -> Self {
        let is_ascii = text.bytes().all(|b| b < 128);
        if is_ascii {
            Self {
                text,
                char_byte_offsets: None,
                total_chars: text.len(),
            }
        } else {
            let mut offsets = Vec::with_capacity(text.len() / 2);
            for (char_idx, (byte_idx, _)) in text.char_indices().enumerate() {
                offsets.push(byte_idx);
                let _ = char_idx;
            }
            offsets.push(text.len()); // sentinel for text end
            let total = offsets.len() - 1;
            Self {
                text,
                char_byte_offsets: Some(offsets),
                total_chars: total,
            }
        }
    }

    pub fn total_chars(&self) -> usize {
        self.total_chars
    }

    pub fn byte_to_char(&self, byte_offset: usize) -> usize {
        if byte_offset >= self.text.len() {
            return self.total_chars;
        }
        match &self.char_byte_offsets {
            None => byte_offset, // pure ASCII: byte offset == char index
            Some(offsets) => {
                // Binary search for exact byte position
                match offsets.binary_search(&byte_offset) {
                    Ok(idx) => idx,
                    Err(idx) => idx.saturating_sub(1),
                }
            }
        }
    }
}

/// Standalone fallback function for one-off byte-to-char mapping.
pub fn byte_to_char_idx(line_text: &str, byte_offset: usize) -> usize {
    if byte_offset >= line_text.len() {
        return line_text.chars().count();
    }
    let mut char_count = 0;
    for (b_idx, _) in line_text.char_indices() {
        if b_idx >= byte_offset {
            return char_count;
        }
        char_count += 1;
    }
    char_count
}

#[derive(Debug, Clone)]
struct CachedLine {
    hash: u64,
    spans: Vec<HighlightSpan>,
}

pub struct Highlighter {
    pub lang: SupportedLanguage,
    parser: Option<Parser>,
    tree: Option<Tree>,
    has_pending_edit: bool,
    cache: RefCell<HashMap<usize, CachedLine>>,
}

impl Highlighter {
    pub fn new(lang: SupportedLanguage) -> Self {
        let mut parser = None;

        if let Some(l) = lang.tree_sitter_language() {
            let mut p = Parser::new();
            if p.set_language(&l).is_ok() {
                parser = Some(p);
            }
        }

        Self {
            lang,
            parser,
            tree: None,
            has_pending_edit: false,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Safely updates the syntax tree from an asynchronous background parsing task,
    /// resetting pending edits and clearing line highlight caches.
    pub fn set_tree(&mut self, tree: Tree) {
        self.tree = Some(tree);
        self.has_pending_edit = false;
        self.cache.borrow_mut().clear();
    }

    /// Returns whether a parsed syntax tree is currently present.
    pub fn has_tree(&self) -> bool {
        self.tree.is_some()
    }

    /// Applies a localized text edit to the existing syntax tree before reparsing.
    ///
    /// Calling this enables Tree-sitter's incremental parsing, allowing unchanged
    /// subtrees to be reused with minimal latency during continuous typing.
    pub fn apply_edit(&mut self, edit: &InputEdit) {
        if let Some(ref mut tree) = self.tree {
            tree.edit(edit);
            self.has_pending_edit = true;
        }
        self.cache.borrow_mut().clear();
    }

    pub fn update_source(&mut self, source: &str) {
        if let Some(ref mut parser) = self.parser {
            let old_tree = if self.has_pending_edit {
                self.tree.as_ref()
            } else {
                None
            };
            self.tree = parser.parse(source, old_tree);
            self.has_pending_edit = false;
        }
        self.cache.borrow_mut().clear();
    }

    /// Updates the syntax tree directly from a `ropey::Rope` using chunk callbacks (`parse_with`),
    /// completely eliminating the 50MB intermediate string allocation and copying overhead.
    pub fn update_source_from_rope(&mut self, rope: &ropey::Rope) {
        if let Some(ref mut parser) = self.parser {
            let old_tree = if self.has_pending_edit {
                self.tree.as_ref()
            } else {
                None
            };
            self.tree = parser.parse_with(
                &mut |byte_offset, _position| {
                    if byte_offset >= rope.len_bytes() {
                        return &[] as &[u8];
                    }
                    let (chunk, chunk_byte_idx, _, _) = rope.chunk_at_byte(byte_offset);
                    let rel = byte_offset - chunk_byte_idx;
                    &chunk.as_bytes()[rel..]
                },
                old_tree,
            );
            self.has_pending_edit = false;
        }
        self.cache.borrow_mut().clear();
    }

    /// Returns the number of cached highlighted lines currently held in memory.
    pub fn cached_line_count(&self) -> usize {
        self.cache.borrow().len()
    }

    pub fn highlight_line(&self, line_text: &str, line_idx: usize) -> Vec<HighlightSpan> {
        if line_text.is_empty() {
            return Vec::new();
        }

        // Fast hash check for line caching
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        line_text.hash(&mut hasher);
        let hash = hasher.finish();

        if let Some(entry) = self.cache.borrow().get(&line_idx) {
            if entry.hash == hash {
                return entry.spans.clone();
            }
        }

        let mut spans = Vec::new();

        match self.lang {
            SupportedLanguage::Markdown => {
                self.highlight_markdown_line(line_text, &mut spans);
            }
            SupportedLanguage::Ini => {
                self.highlight_ini_line(line_text, &mut spans);
            }
            SupportedLanguage::PlainText => {
                spans.push(HighlightSpan {
                    start_col: 0,
                    end_col: line_text.chars().count(),
                    token_type: TokenType::Default,
                });
            }
            _ => {
                if let Some(ref tree) = self.tree {
                    self.highlight_line_from_tree(tree, line_text, line_idx, &mut spans);
                    if spans.is_empty() {
                        self.fallback_lexical_highlight(line_text, &mut spans);
                    }
                } else {
                    self.fallback_lexical_highlight(line_text, &mut spans);
                }
            }
        }

        let mut cache = self.cache.borrow_mut();
        if cache.len() >= 2000 {
            cache.clear();
        }
        cache.insert(
            line_idx,
            CachedLine {
                hash,
                spans: spans.clone(),
            },
        );

        spans
    }

    fn highlight_line_from_tree(
        &self,
        tree: &Tree,
        line_text: &str,
        line_idx: usize,
        out: &mut Vec<HighlightSpan>,
    ) {
        let root = tree.root_node();
        let mapper = ByteCharMapper::new(line_text);

        fn visit_node(
            node: Node,
            target_line: usize,
            mapper: &ByteCharMapper,
            lang: SupportedLanguage,
            out: &mut Vec<HighlightSpan>,
        ) {
            let start = node.start_position();
            let end = node.end_position();

            if start.row > target_line || end.row < target_line {
                return;
            }

            let token_type = classify_node(lang, &node);

            if let Some(tt) = token_type {
                let start_byte = if start.row == target_line {
                    start.column
                } else {
                    0
                };
                let end_byte = if end.row == target_line {
                    end.column
                } else {
                    mapper.text.len()
                };

                let start_col = mapper.byte_to_char(start_byte);
                let end_col = mapper.byte_to_char(end_byte);

                if start_col < end_col {
                    out.push(HighlightSpan {
                        start_col,
                        end_col,
                        token_type: tt,
                    });
                }
            }

            let is_leaf_like = matches!(
                token_type,
                Some(
                    TokenType::Comment
                        | TokenType::NumberLit
                        | TokenType::Keyword
                        | TokenType::StringLit
                )
            );

            if !is_leaf_like {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    visit_node(child, target_line, mapper, lang, out);
                }
            }
        }

        visit_node(root, line_idx, &mapper, self.lang, out);
        out.sort_by_key(|s| s.start_col);
    }

    fn fallback_lexical_highlight(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let mapper = ByteCharMapper::new(line);
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            let leading_bytes = line.len() - trimmed.len();
            out.push(HighlightSpan {
                start_col: mapper.byte_to_char(leading_bytes),
                end_col: mapper.total_chars(),
                token_type: TokenType::Comment,
            });
            return;
        }

        let words = [
            ("fn", TokenType::Keyword),
            ("def", TokenType::Keyword),
            ("function", TokenType::Keyword),
            ("let", TokenType::Keyword),
            ("const", TokenType::Keyword),
            ("var", TokenType::Keyword),
            ("mut", TokenType::Keyword),
            ("pub", TokenType::Keyword),
            ("struct", TokenType::Keyword),
            ("class", TokenType::Keyword),
            ("enum", TokenType::Keyword),
            ("impl", TokenType::Keyword),
            ("use", TokenType::Keyword),
            ("import", TokenType::Keyword),
            ("from", TokenType::Keyword),
            ("export", TokenType::Keyword),
            ("return", TokenType::Keyword),
            ("if", TokenType::Keyword),
            ("else", TokenType::Keyword),
            ("for", TokenType::Keyword),
            ("while", TokenType::Keyword),
            ("true", TokenType::NumberLit),
            ("false", TokenType::NumberLit),
            ("null", TokenType::NumberLit),
            ("None", TokenType::NumberLit),
        ];

        let mut in_quote = false;
        let mut quote_start_char = 0;

        for (char_idx, c) in line.chars().enumerate() {
            if c == '"' || c == '\'' {
                if in_quote {
                    out.push(HighlightSpan {
                        start_col: quote_start_char,
                        end_col: char_idx + 1,
                        token_type: TokenType::StringLit,
                    });
                    in_quote = false;
                } else {
                    in_quote = true;
                    quote_start_char = char_idx;
                }
            }
        }

        if in_quote {
            out.push(HighlightSpan {
                start_col: quote_start_char,
                end_col: mapper.total_chars(),
                token_type: TokenType::StringLit,
            });
        }

        for (word, tt) in words {
            let mut start_byte = 0;
            while let Some(pos) = line[start_byte..].find(word) {
                let actual_byte = start_byte + pos;
                let before = if actual_byte > 0 {
                    line[..actual_byte].chars().next_back()
                } else {
                    None
                };
                let after = line[actual_byte + word.len()..].chars().next();

                let is_boundary = before.is_none_or(|c| !c.is_alphanumeric() && c != '_')
                    && after.is_none_or(|c| !c.is_alphanumeric() && c != '_');

                if is_boundary {
                    let start_col = mapper.byte_to_char(actual_byte);
                    let end_col = mapper.byte_to_char(actual_byte + word.len());
                    out.push(HighlightSpan {
                        start_col,
                        end_col,
                        token_type: tt,
                    });
                }
                start_byte = actual_byte + word.len();
            }
        }

        out.sort_by_key(|s| s.start_col);
    }

    fn highlight_markdown_line(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let mapper = ByteCharMapper::new(line);
        let trimmed = line.trim_start();
        let leading_bytes = line.len() - trimmed.len();
        let leading_col = mapper.byte_to_char(leading_bytes);
        let total_chars = mapper.total_chars();

        if trimmed.starts_with('#') {
            out.push(HighlightSpan {
                start_col: leading_col,
                end_col: total_chars,
                token_type: TokenType::Heading,
            });
        } else if trimmed.starts_with("```") {
            out.push(HighlightSpan {
                start_col: 0,
                end_col: total_chars,
                token_type: TokenType::Keyword,
            });
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("> ")
        {
            out.push(HighlightSpan {
                start_col: leading_col,
                end_col: (leading_col + 2).min(total_chars),
                token_type: TokenType::Operator,
            });
        }
    }

    fn highlight_ini_line(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let mapper = ByteCharMapper::new(line);
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with(';') {
            let leading_bytes = line.len() - line.trim_start().len();
            out.push(HighlightSpan {
                start_col: mapper.byte_to_char(leading_bytes),
                end_col: mapper.total_chars(),
                token_type: TokenType::Comment,
            });
            return;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let start_byte = line.find('[').unwrap_or(0);
            let end_byte = line.rfind(']').map_or(line.len(), |p| p + 1);
            out.push(HighlightSpan {
                start_col: mapper.byte_to_char(start_byte),
                end_col: mapper.byte_to_char(end_byte),
                token_type: TokenType::Heading,
            });
            return;
        }

        if let Some((k, v)) = line.split_once('=') {
            let k_trim = k.trim_start();
            let k_indent_bytes = k.len() - k_trim.len();
            let k_indent_col = mapper.byte_to_char(k_indent_bytes);
            let k_end_bytes = k_indent_bytes + k_trim.trim_end().len();
            let k_end_col = mapper.byte_to_char(k_end_bytes);
            out.push(HighlightSpan {
                start_col: k_indent_col,
                end_col: k_end_col,
                token_type: TokenType::Variable,
            });

            let eq_pos_bytes = k.len();
            out.push(HighlightSpan {
                start_col: mapper.byte_to_char(eq_pos_bytes),
                end_col: mapper.byte_to_char(eq_pos_bytes + 1),
                token_type: TokenType::Operator,
            });

            let val_trimmed = v.trim();
            if !val_trimmed.is_empty() {
                let v_start_bytes = eq_pos_bytes + 1 + (v.len() - v.trim_start().len());
                let v_end_bytes = v_start_bytes + val_trimmed.len();
                let tt = if val_trimmed.starts_with('"') || val_trimmed.starts_with('\'') {
                    TokenType::StringLit
                } else if val_trimmed.chars().all(|c| c.is_numeric() || c == '.')
                    || val_trimmed.eq_ignore_ascii_case("true")
                    || val_trimmed.eq_ignore_ascii_case("false")
                {
                    TokenType::NumberLit
                } else {
                    TokenType::StringLit
                };
                out.push(HighlightSpan {
                    start_col: mapper.byte_to_char(v_start_bytes),
                    end_col: mapper.byte_to_char(v_end_bytes),
                    token_type: tt,
                });
            }
        }
    }
}

fn classify_node(lang: SupportedLanguage, node: &Node) -> Option<TokenType> {
    let kind = node.kind();

    // Universal comment handling
    if kind == "comment" || kind == "line_comment" || kind == "block_comment" {
        return Some(TokenType::Comment);
    }

    // Universal string handling
    if kind == "string"
        || kind == "string_literal"
        || kind == "raw_string_literal"
        || kind == "char_literal"
        || kind == "single_quote_string"
        || kind == "double_quote_string"
        || kind == "string_scalar"
        || kind == "string_content"
    {
        return Some(TokenType::StringLit);
    }

    // Universal number & boolean handling
    if kind == "number"
        || kind == "integer"
        || kind == "float"
        || kind == "integer_literal"
        || kind == "float_literal"
        || kind == "number_literal"
        || kind == "integer_scalar"
        || kind == "float_scalar"
        || kind == "boolean_scalar"
        || kind == "boolean_literal"
        || kind == "true"
        || kind == "false"
        || kind == "null"
        || kind == "nil"
        || kind == "None"
    {
        return Some(TokenType::NumberLit);
    }

    match lang {
        SupportedLanguage::Rust => match kind {
            "fn" | "let" | "mut" | "struct" | "enum" | "impl" | "use" | "pub" | "crate" | "mod"
            | "match" | "if" | "else" | "return" | "while" | "for" | "in" | "loop" | "where"
            | "as" | "break" | "continue" | "self" | "super" | "type" | "const" | "static"
            | "trait" | "async" | "await" | "unsafe" => Some(TokenType::Keyword),
            "identifier"
                if node.parent().is_some_and(|p| {
                    p.kind() == "call_expression" || p.kind() == "function_item"
                }) =>
            {
                Some(TokenType::Function)
            }
            "type_identifier" | "primitive_type" => Some(TokenType::TypeName),
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | "+=" | "-=" | "=>" | "->" => {
                Some(TokenType::Operator)
            }
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "::" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::Python => match kind {
            "def" | "class" | "return" | "if" | "else" | "elif" | "for" | "while" | "try"
            | "except" | "finally" | "with" | "as" | "import" | "from" | "yield" | "async"
            | "await" | "lambda" | "pass" | "break" | "continue" | "in" | "is" | "not" | "and"
            | "or" | "global" | "nonlocal" => Some(TokenType::Keyword),
            "identifier"
                if node
                    .parent()
                    .is_some_and(|p| p.kind() == "call" || p.kind() == "function_definition") =>
            {
                Some(TokenType::Function)
            }
            "type" => Some(TokenType::TypeName),
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "->"
            | "+=" | "-=" | "*=" | "/=" => Some(TokenType::Operator),
            "{" | "}" | "(" | ")" | "[" | "]" | ":" | "," | "." => Some(TokenType::Punctuation),
            _ => None,
        },
        SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => match kind {
            "function" | "const" | "let" | "var" | "return" | "if" | "else" | "for" | "while"
            | "do" | "switch" | "case" | "default" | "break" | "continue" | "import" | "from"
            | "export" | "class" | "extends" | "new" | "this" | "super" | "try" | "catch"
            | "finally" | "throw" | "typeof" | "instanceof" | "async" | "await" | "yield"
            | "type" | "interface" => Some(TokenType::Keyword),
            "identifier"
                if node.parent().is_some_and(|p| {
                    p.kind() == "call_expression"
                        || p.kind() == "function_declaration"
                        || p.kind() == "method_definition"
                }) =>
            {
                Some(TokenType::Function)
            }
            "type_identifier" | "predefined_type" => Some(TokenType::TypeName),
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "===" | "!=" | "!==" | "<" | ">" | "<="
            | ">=" | "&&" | "||" | "!" | "=>" => Some(TokenType::Operator),
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::C | SupportedLanguage::Cpp => match kind {
            "if" | "else" | "for" | "while" | "do" | "switch" | "case" | "default" | "break"
            | "continue" | "return" | "goto" | "struct" | "union" | "enum" | "typedef"
            | "sizeof" | "static" | "const" | "volatile" | "extern" | "inline" | "class"
            | "namespace" | "template" | "public" | "private" | "protected" | "virtual"
            | "override" | "new" | "delete" => Some(TokenType::Keyword),
            "primitive_type" | "type_identifier" => Some(TokenType::TypeName),
            "identifier"
                if node.parent().is_some_and(|p| {
                    p.kind() == "call_expression" || p.kind() == "function_declarator"
                }) =>
            {
                Some(TokenType::Function)
            }
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "&&"
            | "||" | "!" | "&" | "|" | "^" | "->" | "::" => Some(TokenType::Operator),
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::Bash => match kind {
            "if" | "then" | "else" | "elif" | "fi" | "for" | "in" | "do" | "done" | "while"
            | "until" | "case" | "esac" | "function" | "select" => Some(TokenType::Keyword),
            "command_name" => Some(TokenType::Function),
            "variable_name" => Some(TokenType::Variable),
            "|" | "||" | "&&" | "&" | ">" | "<" | ">>" | "<<" | "=" => Some(TokenType::Operator),
            _ => None,
        },
        SupportedLanguage::Fish => match kind {
            "function" | "end" | "if" | "else" | "for" | "in" | "while" | "switch" | "case"
            | "and" | "or" | "not" | "set" | "return" | "break" | "continue" => {
                Some(TokenType::Keyword)
            }
            "word"
                if node.parent().is_some_and(|p| {
                    p.kind() == "command" || p.kind() == "function_definition"
                }) =>
            {
                Some(TokenType::Function)
            }
            "variable_name" => Some(TokenType::Variable),
            "|" | "&" | ">" | "<" | ">>" => Some(TokenType::Operator),
            _ => None,
        },
        SupportedLanguage::Toml => match kind {
            "bare_key" => {
                if node
                    .parent()
                    .is_some_and(|p| p.kind() == "table" || p.kind() == "table_array_element")
                {
                    Some(TokenType::Heading)
                } else {
                    Some(TokenType::Variable)
                }
            }
            "=" => Some(TokenType::Operator),
            "[" | "]" | "{" | "}" | "," => Some(TokenType::Punctuation),
            _ => None,
        },
        SupportedLanguage::Yaml => match kind {
            "flow_node" if node.next_sibling().is_some_and(|s| s.kind() == ":") => {
                Some(TokenType::Variable)
            }
            ":" | "-" | "?" => Some(TokenType::Operator),
            _ => None,
        },
        SupportedLanguage::Json => match kind {
            "string"
                if node.parent().is_some_and(|p| {
                    p.kind() == "pair" && p.start_position() == node.start_position()
                }) =>
            {
                Some(TokenType::Variable)
            }
            ":" => Some(TokenType::Operator),
            "{" | "}" | "[" | "]" | "," => Some(TokenType::Punctuation),
            _ => None,
        },
        SupportedLanguage::Ini => match kind {
            "section_name" => Some(TokenType::Heading),
            "setting_name" => Some(TokenType::Variable),
            "=" => Some(TokenType::Operator),
            "[" | "]" => Some(TokenType::Punctuation),
            _ => None,
        },
        SupportedLanguage::Markdown | SupportedLanguage::PlainText => None,
    }
}
