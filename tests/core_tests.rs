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

#[test]
fn test_japanese_text_and_width() {
    use unicode_width::UnicodeWidthChar;

    let mut buf = TextBuffer::new("こんにちは世界！\nHello World");
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.line_text(0), Some("こんにちは世界！".to_string()));

    // Test CJK character display widths
    let c = 'あ';
    assert_eq!(c.width().unwrap_or(1), 2);
    let a = 'a';
    assert_eq!(a.width().unwrap_or(1), 1);

    // Insert Japanese text into buffer
    buf.cursor = (0, 5); // after "こんにちは"
    buf.insert_str("、素晴らしき");
    assert_eq!(buf.line_text(0), Some("こんにちは、素晴らしき世界！".to_string()));

    // Undo Japanese insertion
    buf.undo();
    assert_eq!(buf.line_text(0), Some("こんにちは世界！".to_string()));

    // Redo Japanese insertion
    buf.redo();
    assert_eq!(buf.line_text(0), Some("こんにちは、素晴らしき世界！".to_string()));
}

#[test]
fn test_editor_pane_saving() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use std::fs;

    let mut pane = EditorPane::new(PaneId::Left, "Untitled");
    // Saving without a file path should fail with an error
    assert!(pane.save_file().is_err());

    // Save as to a new temporary path
    let temp_dir = std::env::temp_dir().join("rooney_test_save");
    let test_file = temp_dir.join("subdir").join("test_save.rs");
    pane.buffer.insert_str("fn test() { println!(\"saved\"); }");

    assert!(pane.save_file_as(&test_file).is_ok());
    assert_eq!(pane.file_name, "test_save.rs");
    assert!(test_file.exists());

    let content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(content, "fn test() { println!(\"saved\"); }");

    // Modify and save again directly
    pane.buffer.insert_str("\n// Added comment");
    assert!(pane.save_file().is_ok());

    let updated_content = fs::read_to_string(&test_file).unwrap();
    assert_eq!(updated_content, "fn test() { println!(\"saved\"); }\n// Added comment");

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_file_tree_navigation() {
    let mut tree = FileTree::new(".");
    let original_root = tree.root.clone();

    // Go to parent directory
    let navigated = tree.go_to_parent();
    assert!(navigated);
    assert_ne!(tree.root, original_root);
    assert!(!tree.items.is_empty());

    // Set root back
    tree.set_root(original_root.clone());
    assert_eq!(tree.root, original_root);
}

#[test]
fn test_app_config_roundtrip() {
    use rooney::config::AppConfig;

    let mut config = AppConfig::default();
    assert_eq!(config.theme, ThemeId::TokyoNight);
    assert_eq!(config.font_size, 14.0);

    config.theme = ThemeId::MatrixGreen;
    config.font_size = 18.0;
    config.opacity = 0.85;
    config.dimming = 0.25;
    config.split_layout = "Split".to_string();
    config.file_tree_visible = false;
    config.ai_enabled = true;
    config.ai_model = "deepseek-coder:6.7b".to_string();

    let toml_str = toml::to_string_pretty(&config).unwrap();
    let loaded: AppConfig = toml::from_str(&toml_str).unwrap();

    assert_eq!(loaded.theme, ThemeId::MatrixGreen);
    assert_eq!(loaded.font_size, 18.0);
    assert_eq!(loaded.opacity, 0.85);
    assert_eq!(loaded.dimming, 0.25);
    assert_eq!(loaded.split_layout, "Split");
    assert!(!loaded.file_tree_visible);
    assert!(loaded.ai_enabled);
    assert_eq!(loaded.ai_model, "deepseek-coder:6.7b");
}

#[test]
fn test_char_advance_ascii_and_cjk() {
    use rooney::ui::canvas_editor::EditorCanvas;

    let font_size = 14.0;
    let ascii_advance = EditorCanvas::char_advance('a', font_size);
    let cjk_advance_hiragana = EditorCanvas::char_advance('あ', font_size);
    let cjk_advance_kanji = EditorCanvas::char_advance('漢', font_size);
    let tab_advance = EditorCanvas::char_advance('\t', font_size);

    // ASCII advance should be exactly 0.60 * font_size
    assert!((ascii_advance - 14.0 * 0.60).abs() < 1e-4);

    // CJK characters must be exactly 1.0 * font_size (not 1.2 * font_size or 2 * 8.4)
    assert!((cjk_advance_hiragana - 14.0).abs() < 1e-4);
    assert!((cjk_advance_kanji - 14.0).abs() < 1e-4);

    // Tab advance is 4 spaces
    assert!((tab_advance - 4.0 * 14.0 * 0.60).abs() < 1e-4);
}

#[test]
fn test_buffer_selection_and_deletion() {
    let mut buf = TextBuffer::new("Hello Beautiful World");
    
    // Select "Beautiful "
    buf.selection_anchor = Some((0, 6));
    buf.cursor = (0, 16);
    assert_eq!(buf.selected_text(), Some("Beautiful ".to_string()));

    // Delete selection
    assert!(buf.delete_selection());
    assert_eq!(buf.full_text(), "Hello World");
    assert_eq!(buf.cursor, (0, 6));
    assert_eq!(buf.selection_anchor, None);

    // Select all
    buf.select_all();
    assert_eq!(buf.selection_anchor, Some((0, 0)));
    assert_eq!(buf.cursor, (0, 11));
    assert_eq!(buf.selected_text(), Some("Hello World".to_string()));
}

#[test]
fn test_numpad_key_resolution() {
    use cosmic::iced::keyboard::key::{Code, Physical};
    use rooney::editor::resolve_numpad_char;

    // Test that numpad codes map to their respective digits and operators
    let numpad_mappings = [
        (Code::Numpad0, "0"),
        (Code::Numpad1, "1"),
        (Code::Numpad2, "2"),
        (Code::Numpad3, "3"),
        (Code::Numpad4, "4"),
        (Code::Numpad5, "5"),
        (Code::Numpad6, "6"),
        (Code::Numpad7, "7"),
        (Code::Numpad8, "8"),
        (Code::Numpad9, "9"),
        (Code::NumpadAdd, "+"),
        (Code::NumpadSubtract, "-"),
        (Code::NumpadMultiply, "*"),
        (Code::NumpadDivide, "/"),
        (Code::NumpadDecimal, "."),
        (Code::NumpadComma, ","),
        (Code::NumpadEqual, "="),
    ];

    for (code, expected) in numpad_mappings {
        let physical = Physical::Code(code);
        let resolved = resolve_numpad_char(&physical);
        assert_eq!(resolved, Some(expected));
    }

    // Verify non-numpad keys (e.g. dedicated navigation, digits, letters) do not match
    let non_numpad = [
        Code::ArrowLeft,
        Code::ArrowRight,
        Code::ArrowUp,
        Code::ArrowDown,
        Code::Home,
        Code::End,
        Code::PageUp,
        Code::PageDown,
        Code::Delete,
        Code::Digit1,
        Code::KeyA,
    ];
    for code in non_numpad {
        let physical = Physical::Code(code);
        assert_eq!(resolve_numpad_char(&physical), None);
    }
}


