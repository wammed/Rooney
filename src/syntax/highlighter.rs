use crate::theme::ThemeConfig;
use cosmic::iced::Color;
use std::path::Path;
use tree_sitter::{Node, Parser, Tree};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedLanguage {
    Rust,
    Toml,
    Markdown,
    PlainText,
}

impl SupportedLanguage {
    pub fn from_path(path: &Path) -> Self {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            "rs" => SupportedLanguage::Rust,
            "toml" => SupportedLanguage::Toml,
            "md" | "markdown" => SupportedLanguage::Markdown,
            _ => SupportedLanguage::PlainText,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            SupportedLanguage::Rust => "Rust",
            SupportedLanguage::Toml => "TOML",
            SupportedLanguage::Markdown => "Markdown",
            SupportedLanguage::PlainText => "Plain Text",
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

        if lang == SupportedLanguage::Rust {
            let mut p = Parser::new();
            if p.set_language(&tree_sitter_rust::LANGUAGE.into()).is_ok() {
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
            SupportedLanguage::Rust => {
                if let Some(ref tree) = self.tree {
                    self.highlight_rust_line_from_tree(tree, line_text, line_idx, &mut spans);
                    if spans.is_empty() {
                        self.fallback_lexical_highlight(line_text, &mut spans);
                    }
                } else {
                    self.fallback_lexical_highlight(line_text, &mut spans);
                }
            }
            SupportedLanguage::Toml => {
                self.highlight_toml_line(line_text, &mut spans);
            }
            SupportedLanguage::Markdown => {
                self.highlight_markdown_line(line_text, &mut spans);
            }
            SupportedLanguage::PlainText => {
                spans.push(HighlightSpan {
                    start_col: 0,
                    end_col: line_text.chars().count(),
                    token_type: TokenType::Default,
                });
            }
        }

        spans
    }

    fn highlight_rust_line_from_tree(
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
            out: &mut Vec<HighlightSpan>,
        ) {
            let start = node.start_position();
            let end = node.end_position();

            if start.row > target_line || end.row < target_line {
                return;
            }

            let kind = node.kind();
            let token_type = match kind {
                "fn" | "let" | "mut" | "struct" | "enum" | "impl" | "use" | "pub" | "crate"
                | "mod" | "match" | "if" | "else" | "return" | "while" | "for" | "in"
                | "loop" | "where" | "as" | "break" | "continue" | "self" | "super"
                | "type" | "const" | "static" | "trait" | "async" | "await" | "unsafe" => {
                    Some(TokenType::Keyword)
                }
                "function_item" | "identifier" if node.parent().map_or(false, |p| p.kind() == "call_expression" || p.kind() == "function_item") => {
                    Some(TokenType::Function)
                }
                "type_identifier" | "primitive_type" => Some(TokenType::TypeName),
                "string_literal" | "raw_string_literal" | "char_literal" => Some(TokenType::StringLit),
                "integer_literal" | "float_literal" | "boolean_literal" => Some(TokenType::NumberLit),
                "line_comment" | "block_comment" => Some(TokenType::Comment),
                "+" | "-" | "*" | "/" | "%" | "=" | "==" | "!=" | "<" | ">" | "<=" | ">="
                | "&&" | "||" | "!" | "&" | "|" | "^" | "<<" | ">>" | "+=" | "-=" | "=>"
                | "->" => Some(TokenType::Operator),
                "{" | "}" | "(" | ")" | "[" | "]" | ";" | ":" | "::" | "," | "." => {
                    Some(TokenType::Punctuation)
                }
                _ => None,
            };

            if let Some(tt) = token_type {
                if start.row == target_line && end.row == target_line {
                    out.push(HighlightSpan {
                        start_col: start.column.min(line_len),
                        end_col: end.column.min(line_len),
                        token_type: tt,
                    });
                }
            }

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                visit_node(child, target_line, line_len, out);
            }
        }

        visit_node(root, line_idx, total_chars, out);
        out.sort_by_key(|s| s.start_col);
    }

    fn fallback_lexical_highlight(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            out.push(HighlightSpan {
                start_col: line.len() - trimmed.len(),
                end_col: line.chars().count(),
                token_type: TokenType::Comment,
            });
            return;
        }

        let words = [
            ("fn", TokenType::Keyword),
            ("let", TokenType::Keyword),
            ("mut", TokenType::Keyword),
            ("pub", TokenType::Keyword),
            ("struct", TokenType::Keyword),
            ("enum", TokenType::Keyword),
            ("impl", TokenType::Keyword),
            ("use", TokenType::Keyword),
            ("mod", TokenType::Keyword),
            ("match", TokenType::Keyword),
            ("if", TokenType::Keyword),
            ("else", TokenType::Keyword),
            ("return", TokenType::Keyword),
            ("loop", TokenType::Keyword),
            ("while", TokenType::Keyword),
            ("for", TokenType::Keyword),
            ("in", TokenType::Keyword),
            ("as", TokenType::Keyword),
            ("true", TokenType::NumberLit),
            ("false", TokenType::NumberLit),
            ("Some", TokenType::TypeName),
            ("None", TokenType::TypeName),
            ("Ok", TokenType::TypeName),
            ("Err", TokenType::TypeName),
        ];

        let mut in_quote = false;
        let mut quote_start = 0;

        for (i, c) in line.char_indices() {
            if c == '"' {
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

                let is_boundary = before.map_or(true, |c| !c.is_alphanumeric() && c != '_')
                    && after.map_or(true, |c| !c.is_alphanumeric() && c != '_');

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

    fn highlight_toml_line(&self, line: &str, out: &mut Vec<HighlightSpan>) {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            out.push(HighlightSpan {
                start_col: 0,
                end_col: line.chars().count(),
                token_type: TokenType::Comment,
            });
            return;
        }

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            out.push(HighlightSpan {
                start_col: 0,
                end_col: line.chars().count(),
                token_type: TokenType::Keyword,
            });
            return;
        }

        if let Some((k, v)) = line.split_once('=') {
            let k_len = k.chars().count();
            out.push(HighlightSpan {
                start_col: 0,
                end_col: k_len,
                token_type: TokenType::Variable,
            });
            out.push(HighlightSpan {
                start_col: k_len,
                end_col: k_len + 1,
                token_type: TokenType::Operator,
            });

            let val_trimmed = v.trim();
            if val_trimmed.starts_with('"') {
                out.push(HighlightSpan {
                    start_col: k_len + 1,
                    end_col: line.chars().count(),
                    token_type: TokenType::StringLit,
                });
            } else {
                out.push(HighlightSpan {
                    start_col: k_len + 1,
                    end_col: line.chars().count(),
                    token_type: TokenType::NumberLit,
                });
            }
        }
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
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("> ") {
            out.push(HighlightSpan {
                start_col: line.len() - trimmed.len(),
                end_col: line.len() - trimmed.len() + 2,
                token_type: TokenType::Operator,
            });
        }
    }
}
