use rooney::editor::buffer::TextBuffer;
use rooney::fs::tree::{FileTypeIcon, FileTree};
use rooney::markdown::renderer::MarkdownDocument;
use rooney::theme::themes::{EditorTheme, ThemeConfig, ThemeId};
use std::path::Path;

#[test]
fn test_text_buffer_basic_operations() {
    let mut buf = TextBuffer::new("Hello World");
    assert_eq!(buf.line_count(), 1);
    assert_eq!(buf.full_text(), "Hello World");

    buf.cursor = (0, 5);
    buf.insert_str(", Rooney");
    assert_eq!(buf.full_text(), "Hello, Rooney World");

    buf.undo();
    assert_eq!(buf.full_text(), "Hello World");

    buf.redo();
    assert_eq!(buf.full_text(), "Hello, Rooney World");
}

#[test]
fn test_text_buffer_multiline_and_cursor() {
    let mut buf = TextBuffer::new("Line 1\nLine 2\nLine 3");
    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.line_text(0), Some("Line 1".to_string()));
    assert_eq!(buf.line_text(1), Some("Line 2".to_string()));
    assert_eq!(buf.line_text(2), Some("Line 3".to_string()));

    buf.cursor = (0, 2);
    buf.move_down(false);
    assert_eq!(buf.cursor, (1, 2));

    buf.cursor = (1, 100);
    buf.clamp_cursor();
    assert_eq!(buf.cursor.0, 1);
    assert!(buf.cursor.1 <= 6);
}

#[test]
fn test_text_buffer_fim_extraction() {
    let mut buf = TextBuffer::new("fn add(a: i32, b: i32) -> i32 {\n    \n}");
    buf.cursor = (1, 4);
    let (prefix, suffix) = buf.get_fim_prefix_suffix(100);

    assert!(prefix.contains("fn add"));
    assert!(suffix.contains('}'));
}

#[test]
fn test_theme_system_20_themes() {
    assert_eq!(ThemeId::ALL.len(), 20);

    for &theme_id in ThemeId::ALL {
        let config = ThemeConfig::for_id(theme_id);
        assert!(!config.name.is_empty());
        assert_eq!(config.id, theme_id);

        let mut theme = EditorTheme::default();
        theme.set_theme(theme_id);
        assert_eq!(theme.config.id, theme_id);

        theme.opacity = 0.85;
        let bg_alpha = theme.background_with_alpha();
        assert!((bg_alpha.a - 0.85).abs() < 0.001);
    }
}

#[test]
fn test_markdown_parsing() {
    let md = "# Title 1\n\nThis is a paragraph.\n\n- Item 1\n- Item 2\n\n```rust\nfn test() {}\n```";
    let doc = MarkdownDocument::parse(md);

    assert!(!doc.blocks.is_empty());
    let has_heading = doc.blocks.iter().any(|b| match b {
        rooney::markdown::renderer::MarkdownBlock::Heading { level: 1, text } => text == "Title 1",
        _ => false,
    });
    assert!(has_heading, "Heading 1 parsed properly");
}

#[test]
fn test_file_type_icons_nerd_font() {
    let rs_icon = FileTypeIcon::for_path(Path::new("src/main.rs"), false, false);
    assert_eq!(rs_icon.glyph, "");

    let md_icon = FileTypeIcon::for_path(Path::new("README.md"), false, false);
    assert_eq!(md_icon.glyph, "");

    let toml_icon = FileTypeIcon::for_path(Path::new("Cargo.toml"), false, false);
    assert_eq!(toml_icon.glyph, "");

    let dir_icon_closed = FileTypeIcon::for_path(Path::new("src"), true, false);
    assert_eq!(dir_icon_closed.glyph, "");

    let dir_icon_open = FileTypeIcon::for_path(Path::new("src"), true, true);
    assert_eq!(dir_icon_open.glyph, "");
}

#[test]
fn test_file_tree_scanning() {
    let tree = FileTree::new(".");
    assert!(!tree.items.is_empty());
}
