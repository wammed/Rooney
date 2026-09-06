use crate::theme::ThemeConfig;
use cosmic::iced::Color;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

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

#[derive(Debug, Clone)]
pub struct HighlightSpan {
    pub start_col: usize,
    pub end_col: usize,
    pub token_type: TokenType,
}

pub struct Highlighter {
    pub lang: SupportedLanguage,
    parser: Option<Parser>,
    tree: Option<Tree>,
}

impl Highlighter {
    pub fn new(lang: SupportedLanguage) -> Self {
        let mut parser = None;

        let ts_lang = match lang {
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
            SupportedLanguage::Ini
            | SupportedLanguage::Markdown
            | SupportedLanguage::PlainText => None,
        };

        if let Some(l) = ts_lang {
            let mut p = Parser::new();
            if p.set_language(&l).is_ok() {
                parser = Some(p);
            }
        }

        Self {
            lang,
            parser,
            tree: None,
        }
    }

    pub fn update_source(&mut self, source: &str) {
        if let Some(ref mut parser) = self.parser {
            self.tree = parser.parse(source, None);
        }
    }

    pub fn highlight_line(&self, line_text: &str, line_idx: usize) -> Vec<HighlightSpan> {
        let mut spans = Vec::new();

        if line_text.is_empty() {
            return spans;
        }

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
        let total_chars = line_text.chars().count();

        fn visit_node(
            node: Node,
            target_line: usize,
            line_len: usize,
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
                if start.row == target_line && end.row == target_line {
                    out.push(HighlightSpan {
                        start_col: start.column.min(line_len),
                        end_col: end.column.min(line_len),
                        token_type: tt,
                    });
                }
            }

            let is_leaf_like = matches!(
                token_type,
                Some(TokenType::Comment | TokenType::NumberLit | TokenType::Keyword)
            );

            if !is_leaf_like {
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    visit_node(child, target_line, line_len, lang, out);
                }
            }
        }

        visit_node(root, line_idx, total_chars, self.lang, out);
        out.sort_by_key(|s| s.start_col);
    }

    fn fallback_lexical_highlight(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") || trimmed.starts_with('#') {
            out.push(HighlightSpan {
                start_col: line.len() - trimmed.len(),
                end_col: line.chars().count(),
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
        let mut quote_start = 0;

        for (i, c) in line.char_indices() {
            if c == '"' || c == '\'' {
                if in_quote {
                    out.push(HighlightSpan {
                        start_col: quote_start,
                        end_col: i + 1,
                        token_type: TokenType::StringLit,
                    });
                    in_quote = false;
                } else {
                    in_quote = true;
                    quote_start = i;
                }
            }
        }

        if in_quote {
            out.push(HighlightSpan {
                start_col: quote_start,
                end_col: line.len(),
                token_type: TokenType::StringLit,
            });
        }

        for (word, tt) in words {
            let mut start = 0;
            while let Some(pos) = line[start..].find(word) {
                let actual_pos = start + pos;
                let before = if actual_pos > 0 {
                    line.chars().nth(actual_pos - 1)
                } else {
                    None
                };
                let after = line.chars().nth(actual_pos + word.len());

                let is_boundary = before.is_none_or(|c| !c.is_alphanumeric() && c != '_')
                    && after.is_none_or(|c| !c.is_alphanumeric() && c != '_');

                if is_boundary {
                    out.push(HighlightSpan {
                        start_col: actual_pos,
                        end_col: actual_pos + word.len(),
                        token_type: tt,
                    });
                }
                start = actual_pos + word.len();
            }
        }

        out.sort_by_key(|s| s.start_col);
    }

    fn highlight_markdown_line(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            out.push(HighlightSpan {
                start_col: line.len() - trimmed.len(),
                end_col: line.chars().count(),
                token_type: TokenType::Heading,
            });
        } else if trimmed.starts_with("```") {
            out.push(HighlightSpan {
                start_col: 0,
                end_col: line.chars().count(),
                token_type: TokenType::Keyword,
            });
        } else if trimmed.starts_with("- ")
            || trimmed.starts_with("* ")
            || trimmed.starts_with("> ")
        {
            out.push(HighlightSpan {
                start_col: line.len() - trimmed.len(),
                end_col: line.len() - trimmed.len() + 2,
                token_type: TokenType::Operator,
            });
        }
    }

    fn highlight_ini_line(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let trimmed = line.trim();
        if trimmed.starts_with('#') || trimmed.starts_with(';') {
            out.push(HighlightSpan {
                start_col: line.len() - line.trim_start().len(),
                end_col: line.chars().count(),
                token_type: TokenType::Comment,
            });
            return;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let start = line.find('[').unwrap_or(0);
            let end = line.rfind(']').map_or(line.len(), |p| p + 1);
            out.push(HighlightSpan {
                start_col: start,
                end_col: end,
                token_type: TokenType::Heading,
            });
            return;
        }

        if let Some((k, v)) = line.split_once('=') {
            let k_trim = k.trim_start();
            let k_indent = k.len() - k_trim.len();
            let k_len = k_indent + k_trim.trim_end().chars().count();
            out.push(HighlightSpan {
                start_col: k_indent,
                end_col: k_len,
                token_type: TokenType::Variable,
            });

            let eq_pos = k.len();
            out.push(HighlightSpan {
                start_col: eq_pos,
                end_col: eq_pos + 1,
                token_type: TokenType::Operator,
            });

            let val_trimmed = v.trim();
            if !val_trimmed.is_empty() {
                let v_start = eq_pos + 1 + (v.len() - v.trim_start().len());
                let v_end = v_start + val_trimmed.chars().count();
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
                    start_col: v_start,
                    end_col: v_end,
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
            "fn" | "let" | "mut" | "struct" | "enum" | "impl" | "use" | "pub" | "crate"
            | "mod" | "match" | "if" | "else" | "return" | "while" | "for" | "in"
            | "loop" | "where" | "as" | "break" | "continue" | "self" | "super"
            | "type" | "const" | "static" | "trait" | "async" | "await" | "unsafe" => {
                Some(TokenType::Keyword)
            }
            "function_item" | "identifier"
                if node
                    .parent()
                    .is_some_and(|p| p.kind() == "call_expression" || p.kind() == "function_item") =>
            {
                Some(TokenType::Function)
            }
            "type_identifier" | "primitive_type" => Some(TokenType::TypeName),
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | "+=" | "-=" | "=>"
            | "->" => Some(TokenType::Operator),
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "::" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::Python => match kind {
            "def" | "class" | "return" | "if" | "else" | "elif" | "for" | "while"
            | "try" | "except" | "finally" | "with" | "as" | "import" | "from"
            | "yield" | "async" | "await" | "lambda" | "pass" | "break" | "continue"
            | "in" | "is" | "not" | "and" | "or" | "global" | "nonlocal" => {
                Some(TokenType::Keyword)
            }
            "identifier"
                if node
                    .parent()
                    .is_some_and(|p| p.kind() == "call" || p.kind() == "function_definition") =>
            {
                Some(TokenType::Function)
            }
            "type" => Some(TokenType::TypeName),
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "->" | "+=" | "-=" | "*=" | "/=" => Some(TokenType::Operator),
            "{" | "}" | "(" | ")" | "[" | "]" | ":" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::JavaScript | SupportedLanguage::TypeScript => match kind {
            "function" | "const" | "let" | "var" | "return" | "if" | "else" | "for"
            | "while" | "do" | "switch" | "case" | "default" | "break" | "continue"
            | "import" | "from" | "export" | "class" | "extends" | "new" | "this"
            | "super" | "try" | "catch" | "finally" | "throw" | "typeof" | "instanceof"
            | "async" | "await" | "yield" | "type" | "interface" => {
                Some(TokenType::Keyword)
            }
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
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "===" | "!=" | "!==" | "<" | ">"
            | "<=" | ">=" | "&&" | "||" | "!" | "=>" => Some(TokenType::Operator),
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::C | SupportedLanguage::Cpp => match kind {
            "if" | "else" | "for" | "while" | "do" | "switch" | "case" | "default"
            | "break" | "continue" | "return" | "goto" | "struct" | "union" | "enum"
            | "typedef" | "sizeof" | "static" | "const" | "volatile" | "extern"
            | "inline" | "class" | "namespace" | "template" | "public" | "private"
            | "protected" | "virtual" | "override" | "new" | "delete" => {
                Some(TokenType::Keyword)
            }
            "primitive_type" | "type_identifier" => Some(TokenType::TypeName),
            "identifier"
                if node.parent().is_some_and(|p| {
                    p.kind() == "call_expression" || p.kind() == "function_declarator"
                }) =>
            {
                Some(TokenType::Function)
            }
            "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
            | "&&" | "||" | "!" | "&" | "|" | "^" | "->" | "::" => {
                Some(TokenType::Operator)
            }
            "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "," | "." => {
                Some(TokenType::Punctuation)
            }
            _ => None,
        },
        SupportedLanguage::Bash => match kind {
            "if" | "then" | "else" | "elif" | "fi" | "for" | "in" | "do" | "done"
            | "while" | "until" | "case" | "esac" | "function" | "select" => {
                Some(TokenType::Keyword)
            }
            "command_name" => Some(TokenType::Function),
            "variable_name" => Some(TokenType::Variable),
            "|" | "||" | "&&" | "&" | ">" | "<" | ">>" | "<<" | "=" => {
                Some(TokenType::Operator)
            }
            _ => None,
        },
        SupportedLanguage::Fish => match kind {
            "function" | "end" | "if" | "else" | "for" | "in" | "while" | "switch"
            | "case" | "and" | "or" | "not" | "set" | "return" | "break" | "continue" => {
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
