use pulldown_cmark::{Event, Parser, Tag, TagEnd};

#[derive(Debug, Clone)]
pub enum MarkdownBlock {
    Heading { level: u32, text: String },
    Paragraph(String),
    CodeBlock { lang: String, code: String },
    ListItem { depth: usize, text: String },
    BlockQuote(String),
    Rule,
}

#[derive(Debug, Clone, Default)]
pub struct MarkdownDocument {
    pub blocks: Vec<MarkdownBlock>,
}

impl MarkdownDocument {
    pub fn parse(source: &str) -> Self {
        let parser = Parser::new(source);
        let mut blocks = Vec::new();

        let mut current_text = String::new();
        let mut in_heading = None;
        let mut in_code_block = None;
        let mut in_list_item = false;
        let mut in_blockquote = false;
        let mut list_depth: usize = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    in_heading = Some(level as u32);
                    current_text.clear();
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some(level) = in_heading.take() {
                        blocks.push(MarkdownBlock::Heading {
                            level,
                            text: current_text.trim().to_string(),
                        });
                        current_text.clear();
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(l) => l.to_string(),
                        pulldown_cmark::CodeBlockKind::Indented => String::new(),
                    };
                    in_code_block = Some(lang);
                    current_text.clear();
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some(lang) = in_code_block.take() {
                        blocks.push(MarkdownBlock::CodeBlock {
                            lang,
                            code: current_text.clone(),
                        });
                        current_text.clear();
                    }
                }
                Event::Start(Tag::List(_)) => {
                    list_depth += 1;
                }
                Event::End(TagEnd::List(_)) => {
                    if list_depth > 0 {
                        list_depth -= 1;
                    }
                }
                Event::Start(Tag::Item) => {
                    in_list_item = true;
                    current_text.clear();
                }
                Event::End(TagEnd::Item) => {
                    if in_list_item {
                        blocks.push(MarkdownBlock::ListItem {
                            depth: list_depth.saturating_sub(1),
                            text: current_text.trim().to_string(),
                        });
                        current_text.clear();
                        in_list_item = false;
                    }
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    in_blockquote = true;
                    current_text.clear();
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if in_blockquote {
                        blocks.push(MarkdownBlock::BlockQuote(current_text.trim().to_string()));
                        current_text.clear();
                        in_blockquote = false;
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    current_text.clear();
                }
                Event::End(TagEnd::Paragraph) => {
                    let text = current_text.trim().to_string();
                    if !text.is_empty() && in_heading.is_none() && in_code_block.is_none() && !in_list_item && !in_blockquote {
                        blocks.push(MarkdownBlock::Paragraph(text));
                        current_text.clear();
                    }
                }
                Event::Text(t) => {
                    current_text.push_str(&t);
                }
                Event::Code(c) => {
                    current_text.push('`');
                    current_text.push_str(&c);
                    current_text.push('`');
                }
                Event::Rule => {
                    blocks.push(MarkdownBlock::Rule);
                }
                Event::SoftBreak | Event::HardBreak => {
                    current_text.push('\n');
                }
                _ => {}
            }
        }

        Self { blocks }
    }
}
