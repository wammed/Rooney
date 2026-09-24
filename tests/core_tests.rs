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
    let doc = MarkdownDocument::parse(md, rooney::config::MarkdownSpec::Gfm);

    assert!(!doc.blocks.is_empty());
    let has_heading = doc.blocks.iter().any(|b| match b {
        rooney::markdown::renderer::MarkdownBlock::Heading { level: 1, spans } => {
            rooney::markdown::renderer::spans_plain_text(spans) == "Title 1"
        }
        _ => false,
    });
    assert!(has_heading, "Heading 1 parsed properly");
}

#[test]
fn test_markdown_spec_commonmark_vs_gfm() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{AlertKind, InlineSpan, MarkdownBlock, spans_plain_text};

    // 1. Tables
    let table_md = "| Col 1 | Col 2 |\n| :--- | ---: |\n| Val A | Val B |";
    let doc_gfm = MarkdownDocument::parse(table_md, MarkdownSpec::Gfm);
    let doc_cm = MarkdownDocument::parse(table_md, MarkdownSpec::CommonMark);

    let gfm_has_table = doc_gfm.blocks.iter().any(|b| matches!(b, MarkdownBlock::Table(_)));
    assert!(gfm_has_table, "GFM must parse table into Table block");

    let cm_has_table = doc_cm.blocks.iter().any(|b| matches!(b, MarkdownBlock::Table(_)));
    assert!(!cm_has_table, "CommonMark must NOT parse table into Table block");

    // 2. Task Lists
    let task_md = "- [ ] Todo item\n- [x] Done item";
    let doc_gfm_tasks = MarkdownDocument::parse(task_md, MarkdownSpec::Gfm);
    let doc_cm_tasks = MarkdownDocument::parse(task_md, MarkdownSpec::CommonMark);

    let gfm_task_statuses: Vec<Option<bool>> = doc_gfm_tasks
        .blocks
        .iter()
        .filter_map(|b| match b {
            MarkdownBlock::ListItem { task_status, .. } => Some(*task_status),
            _ => None,
        })
        .collect();
    assert_eq!(gfm_task_statuses, vec![Some(false), Some(true)]);

    let cm_task_statuses: Vec<Option<bool>> = doc_cm_tasks
        .blocks
        .iter()
        .filter_map(|b| match b {
            MarkdownBlock::ListItem { task_status, .. } => Some(*task_status),
            _ => None,
        })
        .collect();
    assert_eq!(cm_task_statuses, vec![None, None]);

    // 3. GitHub Alerts
    let alert_md = "> [!NOTE]\n> This is an alert callout.";
    let doc_gfm_alert = MarkdownDocument::parse(alert_md, MarkdownSpec::Gfm);
    let doc_cm_alert = MarkdownDocument::parse(alert_md, MarkdownSpec::CommonMark);

    let gfm_has_alert = doc_gfm_alert.blocks.iter().any(|b| matches!(b, MarkdownBlock::Alert { kind: AlertKind::Note, .. }));
    assert!(gfm_has_alert, "GFM should parse [!NOTE] as Alert");

    let cm_has_alert = doc_cm_alert.blocks.iter().any(|b| matches!(b, MarkdownBlock::Alert { .. }));
    assert!(!cm_has_alert, "CommonMark should treat [!NOTE] as standard BlockQuote");

    // 4. Strikethrough
    let strike_md = "This is ~~deleted~~ text.";
    let doc_gfm_strike = MarkdownDocument::parse(strike_md, MarkdownSpec::Gfm);
    let doc_cm_strike = MarkdownDocument::parse(strike_md, MarkdownSpec::CommonMark);

    let gfm_has_strikethrough = doc_gfm_strike.blocks.iter().any(|b| match b {
        MarkdownBlock::Paragraph(spans) => spans.iter().any(|s| matches!(s, InlineSpan::Strikethrough(t) if t == "deleted")),
        _ => false,
    });
    assert!(gfm_has_strikethrough, "GFM parses strikethrough into InlineSpan::Strikethrough");

    let cm_strike_text = doc_cm_strike.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(spans) => Some(spans_plain_text(spans)),
        _ => None,
    }).unwrap_or_default();
    assert_eq!(cm_strike_text, "This is ~~deleted~~ text.");

    // 5. breaks: false (soft line breaks do not form <br>)
    let break_md = "First line\nSecond line";
    let doc_gfm_break = MarkdownDocument::parse(break_md, MarkdownSpec::Gfm);
    let doc_cm_break = MarkdownDocument::parse(break_md, MarkdownSpec::CommonMark);

    let gfm_break_text = doc_gfm_break.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(spans) => Some(spans_plain_text(spans)),
        _ => None,
    }).unwrap_or_default();
    let cm_break_text = doc_cm_break.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(spans) => Some(spans_plain_text(spans)),
        _ => None,
    }).unwrap_or_default();

    assert_eq!(gfm_break_text, "First line Second line");
    assert_eq!(cm_break_text, "First line Second line");
}

#[test]
fn test_gfm_rich_inline_elements_and_image_fallback() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{InlineSpan, MarkdownBlock};

    let md = "Here is **bold**, *italic*, `inline code`, [a link](https://example.com), and ![Rooney Logo](https://example.com/logo.png).";
    let doc = MarkdownDocument::parse(md, MarkdownSpec::Gfm);

    assert_eq!(doc.blocks.len(), 1);
    match &doc.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Bold(t) if t == "bold")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Italic(t) if t == "italic")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Code(t) if t == "inline code")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Link { text, url } if text == "a link" && url == "https://example.com")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::ImageFallback { alt, url } if alt == "Rooney Logo" && url == "https://example.com/logo.png")));
        }
        _ => panic!("Expected Paragraph block"),
    }
}

#[test]
fn test_markdown_html_stripping_and_br_fallback() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{InlineSpan, MarkdownBlock};

    let md = "<div align=\"center\">\n  <h1>Rooney Editor</h1>\n  <p>First line<br/>Second line</p>\n  <img src=\"banner.png\" alt=\"Banner Image\" />\n</div>";
    let doc = MarkdownDocument::parse(md, MarkdownSpec::Gfm);

    let full_text: String = doc.blocks.iter().map(|b| b.plain_text()).collect::<Vec<_>>().join(" ");
    assert!(full_text.contains("Rooney Editor"), "Stripped HTML retains header text");
    assert!(full_text.contains("First line\nSecond line"), "<br/> converted to newline");
    let has_img_fallback = doc.blocks.iter().any(|b| match b {
        MarkdownBlock::Paragraph(spans) => spans.iter().any(|s| matches!(s, InlineSpan::ImageFallback { alt, url } if alt == "Banner Image" && url == "banner.png")),
        _ => false,
    });
    assert!(has_img_fallback, "HTML <img> tags safely convert to ImageFallback");
}

#[test]
fn test_gfm_footnotes_parsing() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{MarkdownBlock, spans_plain_text};

    let md = "Here is a statement with a footnote[^1].\n\nAnother paragraph.\n\n[^1]: Footnote details here.";
    let doc = MarkdownDocument::parse(md, MarkdownSpec::Gfm);

    let has_footnote_ref = doc.blocks.iter().any(|b| match b {
        MarkdownBlock::Paragraph(spans) => spans_plain_text(spans).contains("[^1]"),
        _ => false,
    });
    assert!(has_footnote_ref, "Document contains footnote reference");

    let footnote_block = doc.blocks.iter().find(|b| matches!(b, MarkdownBlock::Footnote { .. }));
    assert!(footnote_block.is_some(), "Footnote block is retained at document end");
    if let Some(MarkdownBlock::Footnote { label, spans }) = footnote_block {
        assert_eq!(label, "1");
        assert_eq!(spans_plain_text(spans), "Footnote details here.");
    }
}


#[test]
fn test_markdown_spec_config_persistence() {
    use rooney::config::{AppConfig, MarkdownSpec};

    let default_config = AppConfig::default();
    assert_eq!(default_config.markdown_spec, MarkdownSpec::Gfm);

    let toml_str = toml::to_string(&default_config).expect("Must serialize AppConfig");
    assert!(toml_str.contains("markdown_spec = \"GFM\""));

    let cm_toml = toml_str.replace("markdown_spec = \"GFM\"", "markdown_spec = \"CommonMark\"");
    let restored: AppConfig = toml::from_str(&cm_toml).expect("Must deserialize AppConfig");
    assert_eq!(restored.markdown_spec, MarkdownSpec::CommonMark);

    // Test lowercase alias deserialization
    let alias_toml = toml_str.replace("markdown_spec = \"GFM\"", "markdown_spec = \"common_mark\"");
    let restored_alias: AppConfig = toml::from_str(&alias_toml).expect("Must deserialize AppConfig alias");
    assert_eq!(restored_alias.markdown_spec, MarkdownSpec::CommonMark);
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

#[test]
fn test_multi_language_detection() {
    use rooney::syntax::SupportedLanguage;

    let cases = [
        ("src/main.rs", SupportedLanguage::Rust),
        ("app.py", SupportedLanguage::Python),
        ("script.pyi", SupportedLanguage::Python),
        ("index.js", SupportedLanguage::JavaScript),
        ("component.jsx", SupportedLanguage::JavaScript),
        ("server.mjs", SupportedLanguage::JavaScript),
        ("main.ts", SupportedLanguage::TypeScript),
        ("App.tsx", SupportedLanguage::TypeScript),
        ("main.c", SupportedLanguage::C),
        ("header.h", SupportedLanguage::C),
        ("main.cpp", SupportedLanguage::Cpp),
        ("util.hpp", SupportedLanguage::Cpp),
        ("deploy.sh", SupportedLanguage::Bash),
        (".bashrc", SupportedLanguage::Bash),
        ("config.fish", SupportedLanguage::Fish),
        ("test.fish", SupportedLanguage::Fish),
        ("Cargo.toml", SupportedLanguage::Toml),
        ("Cargo.lock", SupportedLanguage::Toml),
        ("docker-compose.yaml", SupportedLanguage::Yaml),
        ("app.yml", SupportedLanguage::Yaml),
        ("package.json", SupportedLanguage::Json),
        (".gitconfig", SupportedLanguage::Ini),
        ("settings.ini", SupportedLanguage::Ini),
        ("server.conf", SupportedLanguage::Ini),
        ("README.md", SupportedLanguage::Markdown),
        ("notes.txt", SupportedLanguage::PlainText),
    ];

    for (file, expected) in cases {
        let detected = SupportedLanguage::from_path(Path::new(file));
        assert_eq!(detected, expected, "Failed detection for {file}");
    }
}

#[test]
fn test_tree_sitter_highlighting_all_languages() {
    use rooney::syntax::{Highlighter, SupportedLanguage};

    let snippets = [
        (SupportedLanguage::Rust, "fn main() { let x = 42; }"),
        (SupportedLanguage::Python, "def greet(name):\n    return f'hello {name}'"),
        (SupportedLanguage::JavaScript, "const add = (a, b) => { return a + b; };"),
        (SupportedLanguage::TypeScript, "function add(a: number, b: number): number { return a + b; }"),
        (SupportedLanguage::C, "int main(int argc, char** argv) { return 0; }"),
        (SupportedLanguage::Cpp, "class App { public: virtual ~App() = default; };"),
        (SupportedLanguage::Bash, "if [ -f $file ]; then\n  echo \"found\"\nfi"),
        (SupportedLanguage::Fish, "function greet\n  echo \"hello $argv\"\nend"),
        (SupportedLanguage::Toml, "[package]\nname = \"rooney\"\nversion = 1"),
        (SupportedLanguage::Yaml, "name: rooney\nversion: 1.0"),
        (SupportedLanguage::Json, "{\n  \"name\": \"rooney\",\n  \"count\": 42\n}"),
        (SupportedLanguage::Ini, "[core]\n  repositoryformatversion = 0"),
    ];

    for (lang, code) in snippets {
        let mut highlighter = Highlighter::new(lang);
        highlighter.update_source(code);
        let spans = highlighter.highlight_line(code.lines().next().unwrap(), 0);
        assert!(!spans.is_empty(), "Spans should not be empty for {lang:?}");
    }
}

#[test]
fn test_search_matches_and_navigation() {
    use rooney::editor::pane::{EditorPane, PaneId};

    let mut pane = EditorPane::new(PaneId::Left, "Test");
    pane.buffer = TextBuffer::new("apple banana apple cherry\nsecond apple line");
    pane.update_search("apple");

    assert_eq!(pane.search_matches.len(), 3);
    assert_eq!(pane.current_match_idx, 0);

    let next = pane.next_search_match();
    assert_eq!(next, Some((0, 13))); // second "apple"
    assert_eq!(pane.current_match_idx, 1);

    let next = pane.next_search_match();
    assert_eq!(next, Some((1, 7))); // third "apple" on line 1
    assert_eq!(pane.current_match_idx, 2);

    let prev = pane.prev_search_match();
    assert_eq!(prev, Some((0, 13)));
    assert_eq!(pane.current_match_idx, 1);
}

#[test]
fn test_word_navigation_and_line_operations() {
    let mut buf = TextBuffer::new("hello world from rooney\nsecond line here");
    buf.cursor = (0, 0);

    buf.move_word_right(false);
    assert_eq!(buf.cursor, (0, 6)); // after "hello "

    buf.move_word_right(false);
    assert_eq!(buf.cursor, (0, 12)); // after "world "

    buf.move_word_left(false);
    assert_eq!(buf.cursor, (0, 6)); // back to "world"

    // Line duplication
    buf.cursor = (0, 2);
    buf.duplicate_line();
    assert_eq!(buf.line_count(), 3);
    assert_eq!(buf.line_text(1), Some("hello world from rooney".to_string()));

    // Line deletion
    buf.cursor = (1, 0);
    buf.delete_line();
    assert_eq!(buf.line_count(), 2);
    assert_eq!(buf.line_text(1), Some("second line here".to_string()));

    // Comment toggle
    buf.cursor = (0, 0);
    buf.toggle_comment("//");
    assert_eq!(buf.line_text(0), Some("// hello world from rooney".to_string()));

    buf.toggle_comment("//");
    assert_eq!(buf.line_text(0), Some("hello world from rooney".to_string()));
}

#[test]
fn test_file_type_icons_extended() {
    let fish_icon = FileTypeIcon::for_path(Path::new("config.fish"), false, false);
    assert_eq!(fish_icon.glyph, "󰈺");

    let yaml_icon = FileTypeIcon::for_path(Path::new("docker-compose.yml"), false, false);
    assert_eq!(yaml_icon.glyph, "");

    let ini_icon = FileTypeIcon::for_path(Path::new("config.ini"), false, false);
    assert_eq!(ini_icon.glyph, "");

    let tsx_icon = FileTypeIcon::for_path(Path::new("App.tsx"), false, false);
    assert_eq!(tsx_icon.glyph, "");
}

#[test]
fn test_pane_tab_management() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use std::fs;

    let mut pane = EditorPane::new(PaneId::Left, "Welcome");
    assert_eq!(pane.tabs.len(), 1);
    assert_eq!(pane.active_tab_idx, 0);
    assert_eq!(pane.file_name, "Welcome");

    // Add a new tab
    pane.new_tab("Script.py");
    assert_eq!(pane.tabs.len(), 2);
    assert_eq!(pane.active_tab_idx, 1);
    assert_eq!(pane.file_name, "Script.py");

    pane.buffer.insert_str("print('hello tab 2')");
    assert_eq!(pane.buffer.full_text(), "print('hello tab 2')");

    // Switch back to tab 0
    pane.select_tab(0);
    assert_eq!(pane.active_tab_idx, 0);
    assert_eq!(pane.file_name, "Welcome");
    assert_eq!(pane.buffer.full_text(), "");

    // Cycle through tabs with next_tab / prev_tab
    pane.next_tab();
    assert_eq!(pane.active_tab_idx, 1);
    assert_eq!(pane.buffer.full_text(), "print('hello tab 2')");

    pane.next_tab();
    assert_eq!(pane.active_tab_idx, 0);

    pane.prev_tab();
    assert_eq!(pane.active_tab_idx, 1);

    // Open file into pane
    let temp_dir = std::env::temp_dir().join("rooney_test_tabs");
    let _ = fs::create_dir_all(&temp_dir);
    let test_file = temp_dir.join("tab_test.rs");
    fs::write(&test_file, "fn tab_func() {}").unwrap();

    // Opening test_file creates a new tab because current tab is dirty/non-empty
    assert!(pane.open_file(&test_file).is_ok());
    assert_eq!(pane.tabs.len(), 3);
    assert_eq!(pane.active_tab_idx, 2);
    assert_eq!(pane.file_name, "tab_test.rs");
    assert_eq!(pane.buffer.full_text(), "fn tab_func() {}");

    // Re-opening the same file switches to existing tab without duplicating
    assert!(pane.open_file(&test_file).is_ok());
    assert_eq!(pane.tabs.len(), 3);
    assert_eq!(pane.active_tab_idx, 2);

    // Close active tab
    pane.close_tab(2);
    assert_eq!(pane.tabs.len(), 2);
    assert_eq!(pane.active_tab_idx, 1);

    // Close remaining tabs until empty -> should fallback to a new Untitled tab
    pane.close_tab(1);
    assert_eq!(pane.tabs.len(), 1);
    pane.close_tab(0);
    assert_eq!(pane.tabs.len(), 1);
    assert_eq!(pane.active_tab_idx, 0);
    assert_eq!(pane.file_name, "Untitled");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_session_serialization_roundtrip() {
    use rooney::config::{AppConfig, PaneSessionInfo, SessionConfig, TabSessionInfo};
    use std::path::PathBuf;

    let session = SessionConfig {
        root_dir: Some(PathBuf::from("/home/user/project")),
        left_pane: PaneSessionInfo {
            active_tab_idx: 1,
            tabs: vec![
                TabSessionInfo {
                    file_path: Some(PathBuf::from("/home/user/project/src/main.rs")),
                    file_name: "main.rs".to_string(),
                    cursor_line: 12,
                    cursor_col: 4,
                },
                TabSessionInfo {
                    file_path: Some(PathBuf::from("/home/user/project/Cargo.toml")),
                    file_name: "Cargo.toml".to_string(),
                    cursor_line: 5,
                    cursor_col: 0,
                },
            ],
        },
        right_pane: Some(PaneSessionInfo {
            active_tab_idx: 0,
            tabs: vec![TabSessionInfo {
                file_path: None,
                file_name: "Untitled".to_string(),
                cursor_line: 0,
                cursor_col: 0,
            }],
        }),
    };

    let config = AppConfig {
        ai_chat_visible: true,
        session,
        ..Default::default()
    };

    let toml_str = toml::to_string_pretty(&config).expect("Failed to serialize AppConfig");
    assert!(toml_str.contains("ai_chat_visible = true"));
    assert!(toml_str.contains("main.rs"));
    assert!(toml_str.contains("Cargo.toml"));

    let deserialized: AppConfig =
        toml::from_str(&toml_str).expect("Failed to deserialize AppConfig");
    assert!(deserialized.ai_chat_visible);
    assert_eq!(
        deserialized.session.root_dir,
        Some(PathBuf::from("/home/user/project"))
    );
    assert_eq!(deserialized.session.left_pane.active_tab_idx, 1);
    assert_eq!(deserialized.session.left_pane.tabs.len(), 2);
    assert_eq!(
        deserialized.session.left_pane.tabs[0].file_path,
        Some(PathBuf::from("/home/user/project/src/main.rs"))
    );
    assert_eq!(deserialized.session.left_pane.tabs[0].cursor_line, 12);
    assert_eq!(deserialized.session.left_pane.tabs[0].cursor_col, 4);
    assert!(deserialized.session.right_pane.is_some());
    let right_pane = deserialized.session.right_pane.unwrap();
    assert_eq!(right_pane.tabs.len(), 1);
    assert_eq!(right_pane.tabs[0].file_path, None);
}

#[test]
fn test_ai_chat_data_structures() {
    use rooney::ai::{ChatMessage, ChatRole};

    let user_msg = ChatMessage {
        role: ChatRole::User,
        content: "Write a Rust hello world".to_string(),
    };
    assert_eq!(user_msg.role, ChatRole::User);
    assert_eq!(user_msg.content, "Write a Rust hello world");

    let assistant_msg = ChatMessage {
        role: ChatRole::Assistant,
        content: "```rust\nfn main() {\n    println!(\"Hello World\");\n}\n```".to_string(),
    };
    assert_eq!(assistant_msg.role, ChatRole::Assistant);

    // Verify code block extraction
    let text = &assistant_msg.content;
    let extracted = if let Some(start) = text.find("```") {
        let after_start = &text[start + 3..];
        if let Some(newline_pos) = after_start.find('\n') {
            let code_start = &after_start[newline_pos + 1..];
            if let Some(end_fence) = code_start.rfind("```") {
                code_start[..end_fence].trim_end().to_string()
            } else {
                text.to_string()
            }
        } else {
            text.to_string()
        }
    } else {
        text.to_string()
    };

    assert_eq!(extracted, "fn main() {\n    println!(\"Hello World\");\n}");
}

#[test]
fn test_ai_chat_streaming_accumulation() {
    use rooney::ai::{ChatMessage, ChatRole, ChatStreamEvent};

    let mut assistant_msg = ChatMessage {
        role: ChatRole::Assistant,
        content: String::new(),
    };

    let events = vec![
        ChatStreamEvent::Chunk("Hello".to_string()),
        ChatStreamEvent::Chunk(", ".to_string()),
        ChatStreamEvent::Chunk("world".to_string()),
        ChatStreamEvent::Chunk("!".to_string()),
        ChatStreamEvent::Done,
    ];

    for ev in events {
        match ev {
            ChatStreamEvent::Chunk(c) => assistant_msg.content.push_str(&c),
            ChatStreamEvent::Done => break,
            ChatStreamEvent::Error(_) => panic!("Unexpected error event"),
        }
    }

    assert_eq!(assistant_msg.content, "Hello, world!");
}

#[test]
fn test_tab_synchronization_on_rename_and_delete() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use std::fs;

    let temp_dir = std::env::temp_dir().join("rooney_test_rename_sync");
    let _ = fs::create_dir_all(&temp_dir);

    let old_file = temp_dir.join("original.rs");
    fs::write(&old_file, "pub fn foo() {}").unwrap();

    let mut pane = EditorPane::new(PaneId::Left, "Welcome");
    assert!(pane.open_file(&old_file).is_ok());
    assert_eq!(pane.file_name, "original.rs");
    assert_eq!(pane.file_path, Some(old_file.clone()));

    // Simulate rename to updated.rs
    let new_file = temp_dir.join("updated.rs");
    fs::rename(&old_file, &new_file).unwrap();

    // Tab synchronization logic
    for tab in &mut pane.tabs {
        if let Some(ref mut tab_path) = tab.file_path {
            if *tab_path == old_file {
                *tab_path = new_file.clone();
                tab.file_name = "updated.rs".to_string();
            }
        }
    }

    assert_eq!(pane.tabs[pane.active_tab_idx].file_name, "updated.rs");
    assert_eq!(
        pane.tabs[pane.active_tab_idx].file_path,
        Some(new_file.clone())
    );

    // Simulate delete
    fs::remove_file(&new_file).unwrap();

    // Tab delete logic
    let mut idx = 0;
    while idx < pane.tabs.len() {
        let matches = pane.tabs[idx]
            .file_path
            .as_ref()
            .map(|p| p == &new_file || p.starts_with(&new_file))
            .unwrap_or(false);
        if matches {
            pane.close_tab(idx);
        } else {
            idx += 1;
        }
    }

    // Should have closed the file tab and fallen back to Untitled
    assert_eq!(pane.tabs.len(), 1);
    assert_eq!(pane.file_name, "Untitled");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_file_tree_context_menu_state_and_right_click() {
    use rooney::app::FileTreeContextMenuState;
    use rooney::ui::file_tree_view::FileTreeMessage;
    use std::path::PathBuf;

    // Test FileTreeMessage variants
    let item_path = PathBuf::from("/tmp/rooney_test/main.rs");
    let right_click_file = FileTreeMessage::RightClick(item_path.clone(), false);
    if let FileTreeMessage::RightClick(path, is_dir) = right_click_file {
        assert_eq!(path, item_path);
        assert!(!is_dir);
    } else {
        panic!("Expected RightClick variant");
    }

    let dir_path = PathBuf::from("/tmp/rooney_test/src");
    let right_click_dir = FileTreeMessage::RightClick(dir_path.clone(), true);
    if let FileTreeMessage::RightClick(path, is_dir) = right_click_dir {
        assert_eq!(path, dir_path);
        assert!(is_dir);
    } else {
        panic!("Expected RightClick variant");
    }

    let right_click_root = FileTreeMessage::RightClickRoot;
    assert!(matches!(right_click_root, FileTreeMessage::RightClickRoot));

    // Test FileTreeContextMenuState
    let menu_file = FileTreeContextMenuState {
        target: Some(item_path.clone()),
        is_dir: false,
        x: 120.0,
        y: 250.0,
    };
    assert_eq!(menu_file.target, Some(item_path));
    assert!(!menu_file.is_dir);
    assert_eq!(menu_file.x, 120.0);
    assert_eq!(menu_file.y, 250.0);

    let menu_root = FileTreeContextMenuState {
        target: None,
        is_dir: true,
        x: 80.0,
        y: 100.0,
    };
    assert!(menu_root.target.is_none());
    assert!(menu_root.is_dir);
}

#[test]
fn test_file_tree_width_and_item_gutter() {
    use std::fs;
    let temp_dir = std::env::temp_dir().join(format!("rooney_tree_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);

    let tree = FileTree::new(&temp_dir);
    // Tree default width should be 280.0
    assert_eq!(tree.width, 280.0);

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_active_header_menu_and_actions() {
    use rooney::app::message::{ActiveHeaderMenu, Message};

    let menu_file = ActiveHeaderMenu::File;
    let menu_edit = ActiveHeaderMenu::Edit;
    let menu_view = ActiveHeaderMenu::View;
    let menu_ai = ActiveHeaderMenu::Ai;

    assert_eq!(menu_file, ActiveHeaderMenu::File);
    assert_ne!(menu_file, menu_edit);
    assert_ne!(menu_view, menu_ai);

    let msg_toggle = Message::ToggleHeaderMenu(ActiveHeaderMenu::File);
    let msg_close = Message::CloseHeaderMenu;

    match msg_toggle {
        Message::ToggleHeaderMenu(m) => assert_eq!(m, ActiveHeaderMenu::File),
        _ => panic!("Expected ToggleHeaderMenu"),
    }
    assert!(matches!(msg_close, Message::CloseHeaderMenu));
}

#[test]
fn test_filename_sanitization_and_path_traversal_guards() {
    use rooney::app::update::is_valid_file_or_folder_name;

    // Valid names
    assert!(is_valid_file_or_folder_name("main.rs"));
    assert!(is_valid_file_or_folder_name("test-123_abc.conf"));
    assert!(is_valid_file_or_folder_name(".gitignore"));
    assert!(is_valid_file_or_folder_name(".env.local"));

    // Path traversal / injection attempts
    assert!(!is_valid_file_or_folder_name("../etc/passwd"));
    assert!(!is_valid_file_or_folder_name(".."));
    assert!(!is_valid_file_or_folder_name("."));
    assert!(!is_valid_file_or_folder_name("/etc/shadow"));
    assert!(!is_valid_file_or_folder_name("sub/folder/file.rs"));
    assert!(!is_valid_file_or_folder_name("dir\\file.txt"));
    assert!(!is_valid_file_or_folder_name(""));
    assert!(!is_valid_file_or_folder_name("   "));
    assert!(!is_valid_file_or_folder_name("file\0null"));
}

#[test]
fn test_sensitive_file_ai_protection() {
    use rooney::app::update::is_sensitive_file;
    use std::path::Path;

    // Sensitive files that must be protected
    assert!(is_sensitive_file(Path::new(".env")));
    assert!(is_sensitive_file(Path::new(".env.local")));
    assert!(is_sensitive_file(Path::new(".env.production")));
    assert!(is_sensitive_file(Path::new("/home/user/.ssh/id_rsa")));
    assert!(is_sensitive_file(Path::new("id_ed25519")));
    assert!(is_sensitive_file(Path::new("server.key")));
    assert!(is_sensitive_file(Path::new("cert.pem")));
    assert!(is_sensitive_file(Path::new("keystore.p12")));
    assert!(is_sensitive_file(Path::new(".git-credentials")));
    assert!(is_sensitive_file(Path::new(".netrc")));
    assert!(is_sensitive_file(Path::new("credentials")));
    assert!(is_sensitive_file(Path::new(".npmrc")));
    assert!(is_sensitive_file(Path::new(".pypirc")));
    assert!(is_sensitive_file(Path::new("kubeconfig")));
    assert!(is_sensitive_file(Path::new("cluster.kubeconfig")));
    assert!(is_sensitive_file(Path::new("app.keystore")));
    assert!(is_sensitive_file(Path::new("release.jks")));
    assert!(is_sensitive_file(Path::new("token")));
    assert!(is_sensitive_file(Path::new("api.token")));
    assert!(is_sensitive_file(Path::new("client_secret.json")));
    assert!(is_sensitive_file(Path::new("/home/user/.aws/credentials")));
    assert!(is_sensitive_file(Path::new("/home/user/.aws/config")));
    assert!(is_sensitive_file(Path::new("/home/user/.kube/config")));

    // Normal files that are allowed for AI
    assert!(!is_sensitive_file(Path::new("main.rs")));
    assert!(!is_sensitive_file(Path::new("index.ts")));
    assert!(!is_sensitive_file(Path::new("README.md")));
    assert!(!is_sensitive_file(Path::new("Cargo.toml")));
    assert!(!is_sensitive_file(Path::new("styles.css")));
}


#[test]
fn test_atomic_save_and_file_size_limits() {
    use rooney::editor::pane::EditorPane;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("rooney_atomic_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let target_file = temp_dir.join("test_save.txt");

    // Test initial write
    let initial_content = "Hello, Atomic World!";
    let res = EditorPane::atomic_write_file(&target_file, initial_content);
    assert!(res.is_ok());
    assert_eq!(fs::read_to_string(&target_file).unwrap(), initial_content);

    // Test atomic replacement
    let updated_content = "Updated content securely replaced!";
    let res2 = EditorPane::atomic_write_file(&target_file, updated_content);
    assert!(res2.is_ok());
    assert_eq!(fs::read_to_string(&target_file).unwrap(), updated_content);

    // Verify no temporary files are left over in the directory
    let entries: Vec<_> = fs::read_dir(&temp_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].file_name(), "test_save.txt");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_rooney_icon_integration() {
    let icon_bytes = include_bytes!("../images/Rooney-matte-icon.svg");
    assert!(!icon_bytes.is_empty());
    assert_eq!(icon_bytes.len(), 25259);

    let icon_str = std::str::from_utf8(icon_bytes).expect("Valid UTF-8 SVG");
    assert!(icon_str.contains("<svg"));
    assert!(icon_str.contains("</svg>"));

    // Verify desktop file config
    if let Some(home) = std::env::var_os("HOME") {
        let home_path = std::path::PathBuf::from(home);
        let desktop_path = home_path.join(".local/share/applications/rooney.desktop");
        if desktop_path.exists() {
            let desktop_content = std::fs::read_to_string(&desktop_path).unwrap();
            assert!(desktop_content.contains("StartupWMClass=rooney"));
            assert!(!desktop_content.contains("StartupWMClass=rooneyk"));
        }

        let icon_path = home_path.join(".local/share/icons/hicolor/scalable/apps/rooney.svg");
        if icon_path.exists() {
            let installed_len = std::fs::metadata(&icon_path).unwrap().len();
            assert_eq!(installed_len, icon_bytes.len() as u64);
        }
    }
}

#[test]
fn test_independent_opacity_settings() {
    let mut theme = EditorTheme::default();
    assert_eq!(theme.file_tree_opacity, 0.95);
    assert_eq!(theme.title_bar_opacity, 0.95);

    theme.file_tree_opacity = 0.5;
    theme.title_bar_opacity = 0.8;

    let sidebar_color = theme.sidebar_with_alpha();
    assert!((sidebar_color.a - 0.5).abs() < 0.001);

    let title_bar_color = theme.title_bar_with_alpha();
    assert!((title_bar_color.a - 0.8).abs() < 0.001);

    // Test Config serialization & deserialization with default and custom values
    let toml_str = r#"
        theme = "TokyoNight"
        font = "JetBrainsMono Nerd Font"
        font_size = 14.0
        opacity = 0.9
        file_tree_opacity = 0.6
        title_bar_opacity = 0.7
        dimming = 0.2
        split_layout = "Split"
        file_tree_visible = true
        ai_enabled = true
        ai_model = "qwen2.5-coder:7b"
    "#;
    let config: rooney::config::AppConfig = toml::from_str(toml_str).expect("Valid config TOML");
    assert_eq!(config.file_tree_opacity, 0.6);
    assert_eq!(config.title_bar_opacity, 0.7);

    theme.opacity = 0.85;
    let bg_color = theme.background_with_alpha();
    assert!((bg_color.a - 0.85).abs() < 0.001);

    // Test backward compatibility when file_tree_opacity & title_bar_opacity are omitted
    let toml_str_legacy = r#"
        theme = "TokyoNight"
        font = "JetBrainsMono Nerd Font"
        font_size = 14.0
        opacity = 0.9
        dimming = 0.2
        split_layout = "Split"
        file_tree_visible = true
        ai_enabled = true
        ai_model = "qwen2.5-coder:7b"
    "#;
    let legacy_config: rooney::config::AppConfig =
        toml::from_str(toml_str_legacy).expect("Valid legacy config TOML");
    assert_eq!(legacy_config.file_tree_opacity, 1.0);
    assert_eq!(legacy_config.title_bar_opacity, 1.0);
}

#[test]
fn test_markdown_preview_opacity_and_background() {
    let mut theme = EditorTheme {
        dimming: 0.0,
        opacity: 0.70,
        ..Default::default()
    };

    let bg_undimmed = theme.background_with_alpha();
    assert!((bg_undimmed.a - 0.70).abs() < 0.001);
    assert!((bg_undimmed.r - theme.config.bg.r).abs() < 0.001);
    assert!((bg_undimmed.g - theme.config.bg.g).abs() < 0.001);
    assert!((bg_undimmed.b - theme.config.bg.b).abs() < 0.001);

    // When dimming is enabled, opacity is strictly preserved, and RGB is darkened
    theme.dimming = 0.5;
    let bg_dimmed = theme.background_with_alpha();
    assert!((bg_dimmed.a - 0.70).abs() < 0.001);
    assert!(bg_dimmed.r < bg_undimmed.r);
    assert!(bg_dimmed.g < bg_undimmed.g);
    assert!(bg_dimmed.b < bg_undimmed.b);

    // MarkdownDocument preview rendering should produce a styled container with alpha
    let doc = MarkdownDocument::parse(
        "# Heading\n\nPreview body with **bold** text.",
        rooney::config::MarkdownSpec::CommonMark,
    );
    let _el: cosmic::Element<'_, ()> = rooney::ui::markdown_view::view_markdown(&doc, &theme, "JetBrainsMono Nerd Font");
}

#[test]
fn test_context_menu_boundary_clamping() {
    let window_w: f32 = 800.0;
    let window_h: f32 = 600.0;
    let menu_w: f32 = 220.0;
    let menu_h: f32 = 240.0;

    let clamp_menu = |cx: f32, cy: f32| -> (f32, f32) {
        let max_w = window_w.max(600.0);
        let max_h = window_h.max(400.0);

        let mut menu_x = cx;
        if menu_x + menu_w > max_w - 20.0 {
            menu_x = cx - menu_w;
        }
        let menu_x = menu_x.clamp(10.0, (max_w - menu_w - 10.0).max(10.0));

        let mut menu_y = cy;
        if menu_y + menu_h > max_h - 36.0 {
            menu_y = cy - menu_h;
        }
        let menu_y = menu_y.clamp(40.0, (max_h - menu_h - 30.0).max(40.0));

        (menu_x, menu_y)
    };

    // Right-click in upper-left area: opens normally downwards and rightwards
    let (x, y) = clamp_menu(50.0, 100.0);
    assert_eq!(x, 50.0);
    assert_eq!(y, 100.0);
    assert!(x + menu_w <= window_w);
    assert!(y + menu_h <= window_h);

    // Right-click at the bottom of the window (e.g. file tree bottom at y=550):
    // must flip upwards and never hide below window bottom
    let (x, y) = clamp_menu(50.0, 550.0);
    assert_eq!(x, 50.0);
    assert_eq!(y, 550.0 - menu_h); // 310.0
    assert!(y + menu_h <= window_h - 30.0);
    assert!(y >= 40.0);

    // Right-click at the far right edge of the window:
    // must flip leftwards
    let (x, _y) = clamp_menu(750.0, 100.0);
    assert_eq!(x, 750.0 - menu_w); // 530.0
    assert!(x + menu_w <= window_w);
    assert!(x >= 10.0);
}

#[test]
fn test_undo_redo_stack_depth_and_performance() {
    use rooney::editor::TextBuffer;

    let mut buffer = TextBuffer::new("start");
    assert_eq!(buffer.undo_stack_len(), 0);
    assert_eq!(buffer.redo_stack_len(), 0);

    // Perform 120 modifications
    for i in 0..120 {
        buffer.insert_str(&format!(" {i}"));
    }

    // Stack should be bounded to 100 items by VecDeque pop_front
    assert_eq!(buffer.undo_stack_len(), 100);

    // Undo all 100 available steps
    for _ in 0..100 {
        buffer.undo();
    }
    assert_eq!(buffer.undo_stack_len(), 0);
    assert_eq!(buffer.redo_stack_len(), 100);

    // Further undo is a no-op
    buffer.undo();
    assert_eq!(buffer.undo_stack_len(), 0);

    // Redo all 100 steps
    for _ in 0..100 {
        buffer.redo();
    }
    assert_eq!(buffer.undo_stack_len(), 100);
    assert_eq!(buffer.redo_stack_len(), 0);
}

#[test]
fn test_buffer_edge_cases() {
    use rooney::editor::TextBuffer;

    // 1. Empty buffer operations
    let mut empty_buf = TextBuffer::new("");
    assert_eq!(empty_buf.line_count(), 1);
    assert_eq!(empty_buf.full_text(), "");
    empty_buf.delete_backspace();
    empty_buf.delete_forward();
    empty_buf.undo();
    empty_buf.redo();
    assert_eq!(empty_buf.cursor, (0, 0));

    empty_buf.insert_char('A');
    assert_eq!(empty_buf.full_text(), "A");
    assert_eq!(empty_buf.cursor, (0, 1));
    empty_buf.undo();
    assert_eq!(empty_buf.full_text(), "");

    // 2. Extremely long line (100,000 characters)
    let long_line: String = "x".repeat(100_000);
    let mut long_buf = TextBuffer::new(&long_line);
    assert_eq!(long_buf.line_count(), 1);
    assert_eq!(long_buf.line_char_count(0), 100_000);

    // Cursor positioning and modification in long line
    long_buf.cursor = (0, 50_000);
    long_buf.insert_str("INSERTED");
    assert_eq!(long_buf.line_char_count(0), 100_008);
    assert_eq!(long_buf.cursor, (0, 50_008));

    long_buf.undo();
    assert_eq!(long_buf.line_char_count(0), 100_000);
    assert!(long_buf.cursor.1 <= 100_000);
}

#[test]
fn test_atomic_config_save() {
    use rooney::config::AppConfig;
    use rooney::editor::EditorTab;
    use rooney::theme::themes::ThemeId;
    use std::fs;

    let temp_dir = std::env::temp_dir().join(format!("rooney_config_test_{}", std::process::id()));
    let _ = fs::create_dir_all(&temp_dir);
    let target_file = temp_dir.join("config.toml");

    let config = AppConfig {
        theme: ThemeId::CyberpunkNeon,
        font_size: 18.0,
        ..Default::default()
    };

    let content = toml::to_string_pretty(&config).unwrap();
    let res = EditorTab::atomic_write_file(&target_file, &content);
    assert!(res.is_ok());

    let loaded_str = fs::read_to_string(&target_file).unwrap();
    let loaded_config: AppConfig = toml::from_str(&loaded_str).unwrap();
    assert_eq!(loaded_config.theme, ThemeId::CyberpunkNeon);
    assert_eq!(loaded_config.font_size, 18.0);

    // Verify temp files are cleaned up
    let entries = fs::read_dir(&temp_dir).unwrap();
    for entry in entries {
        let name = entry.unwrap().file_name().to_string_lossy().to_string();
        assert!(!name.starts_with(".config.toml.tmp"), "Temp file was left behind: {}", name);
    }
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_editor_auto_scroll_flag_and_coordinates() {
    use rooney::editor::pane::{EditorPane, PaneId};

    let mut pane = EditorPane::new(PaneId::Left, "Test");
    // Default pane creation should request cursor focus / visibility
    assert!(pane.needs_scroll_to_cursor.get());

    // Consuming / drawing frame clears flag
    pane.needs_scroll_to_cursor.set(false);
    assert!(!pane.needs_scroll_to_cursor.get());

    // Moving cursor explicitly via keybinding or selection marks cursor moved
    pane.mark_cursor_moved();
    assert!(pane.needs_scroll_to_cursor.get());

    // Simulating user mouse wheel scroll disables auto-scroll until next cursor movement
    pane.needs_scroll_to_cursor.set(false);
    assert!(!pane.needs_scroll_to_cursor.get());

    // Typing / modifying buffer sets needs_scroll_to_cursor back to true
    pane.buffer.insert_char('a');
    pane.on_content_changed();
    assert!(pane.needs_scroll_to_cursor.get());

    // Switching to a new tab initializes needs_scroll_to_cursor to true
    pane.needs_scroll_to_cursor.set(false);
    pane.new_tab("New Document");
    assert!(pane.needs_scroll_to_cursor.get());
}

#[test]
fn test_scrollbar_proportions_and_thumb_mapping() {
    let line_height = 24.0f32;
    let bounds_height = 600.0f32;
    let total_rows = 100usize;
    let total_content_height = (total_rows as f32) * line_height; // 2400.0
    let max_scroll = (total_content_height - bounds_height).max(0.0); // 1800.0

    // 1. Proportional thumb height calculation
    let thumb_height = ((bounds_height / total_content_height) * bounds_height)
        .max(28.0)
        .min(bounds_height);
    assert_eq!(thumb_height, 150.0);

    let max_thumb_y = bounds_height - thumb_height; // 450.0
    assert_eq!(max_thumb_y, 450.0);

    // 2. Thumb y position mapping at top, middle, and bottom
    let scroll_top = 0.0f32;
    let thumb_top = (scroll_top / max_scroll) * max_thumb_y;
    assert_eq!(thumb_top, 0.0);

    let scroll_mid = 900.0f32;
    let thumb_mid = (scroll_mid / max_scroll) * max_thumb_y;
    assert_eq!(thumb_mid, 225.0);

    let scroll_bot = 1800.0f32;
    let thumb_bot = (scroll_bot / max_scroll) * max_thumb_y;
    assert_eq!(thumb_bot, 450.0);

    // 3. Margin auto-scroll calculation when navigating down
    let margin = (2.0 * line_height).min(bounds_height * 0.25).max(0.0); // 48.0
    assert_eq!(margin, 48.0);

    // Cursor at row 95: cursor_top = 2280.0, cursor_bottom = 2304.0
    let cursor_v_idx = 95usize;
    let cursor_top = (cursor_v_idx as f32) * line_height;
    let cursor_bottom = cursor_top + line_height;
    let cur_scroll = 1000.0f32; // Screen currently showing lines ~41-66
    let mut target_scroll = cur_scroll;

    if cursor_top < target_scroll + margin {
        target_scroll = (cursor_top - margin).max(0.0);
    } else if cursor_bottom > target_scroll + bounds_height - margin {
        target_scroll = (cursor_bottom + margin - bounds_height).max(0.0);
    }
    target_scroll = target_scroll.clamp(0.0, max_scroll);
    // 2304.0 + 48.0 - 600.0 = 1752.0
    assert_eq!(target_scroll, 1752.0);

    // 4. Margin auto-scroll calculation when navigating up to row 5
    let cursor_v_idx_up = 5usize;
    let cursor_top_up = (cursor_v_idx_up as f32) * line_height; // 120.0
    let cursor_bottom_up = cursor_top_up + line_height; // 144.0
    let cur_scroll_down = 1500.0f32;
    let mut target_scroll_up = cur_scroll_down;

    if cursor_top_up < target_scroll_up + margin {
        target_scroll_up = (cursor_top_up - margin).max(0.0);
    } else if cursor_bottom_up > target_scroll_up + bounds_height - margin {
        target_scroll_up = (cursor_bottom_up + margin - bounds_height).max(0.0);
    }
    target_scroll_up = target_scroll_up.clamp(0.0, max_scroll);
    // (120.0 - 48.0).max(0.0) = 72.0
    assert_eq!(target_scroll_up, 72.0);

    // 5. Minimum thumb height clamping on very long files (50,000 lines)
    let huge_content_height = 50_000.0 * line_height;
    let huge_thumb_height = ((bounds_height / huge_content_height) * bounds_height)
        .max(28.0)
        .min(bounds_height);
    assert_eq!(huge_thumb_height, 28.0);
}

#[test]
fn test_cli_flags_and_path_resolution() {
    use rooney::app::flags::{parse_path_or_uri, AppFlags};
    use std::path::PathBuf;

    // 1. Standard file:// URI
    let uri = "file:///tmp/rooney_test.txt";
    let parsed = parse_path_or_uri(uri).expect("Should parse file:// URI");
    assert_eq!(parsed, PathBuf::from("/tmp/rooney_test.txt"));

    // 2. URI with percent-encoded spaces and characters
    let uri_space = "file:///tmp/My%20Documents/test%20file.txt";
    let parsed_space = parse_path_or_uri(uri_space).expect("Should parse percent-encoded URI");
    assert_eq!(parsed_space, PathBuf::from("/tmp/My Documents/test file.txt"));

    // 3. URI with percent-encoded Japanese (UTF-8) characters
    let uri_cjk = "file:///tmp/%E3%83%86%E3%82%B9%E3%83%88.txt";
    let parsed_cjk = parse_path_or_uri(uri_cjk).expect("Should parse percent-encoded CJK URI");
    assert_eq!(parsed_cjk, PathBuf::from("/tmp/テスト.txt"));

    // 4. URI with localhost authority
    let uri_localhost = "file://localhost/tmp/local_file.txt";
    let parsed_localhost = parse_path_or_uri(uri_localhost).expect("Should parse file://localhost URI");
    assert_eq!(parsed_localhost, PathBuf::from("/tmp/local_file.txt"));

    // 5. Absolute standard path
    let abs = "/var/log/syslog";
    let parsed_abs = parse_path_or_uri(abs).expect("Should parse absolute path");
    assert_eq!(parsed_abs, PathBuf::from("/var/log/syslog"));

    // 6. Relative path resolution against current directory
    let rel = "src/main.rs";
    let parsed_rel = parse_path_or_uri(rel).expect("Should parse relative path");
    assert!(parsed_rel.is_absolute());
    assert!(parsed_rel.ends_with("src/main.rs"));

    // 7. Normalization of .. and .
    let with_dots = "/tmp/a/../b/./c.txt";
    let parsed_dots = parse_path_or_uri(with_dots).expect("Should parse dots");
    assert_eq!(parsed_dots, PathBuf::from("/tmp/b/c.txt"));

    // 8. Empty input
    assert_eq!(parse_path_or_uri(""), None);
    assert_eq!(parse_path_or_uri("   "), None);

    // 9. AppFlags::from_args skips options and collects files
    let args = vec![
        "-h".to_string(),
        "--help".to_string(),
        "-v".to_string(),
        "--version".to_string(),
        "--".to_string(),
        "file:///tmp/doc.txt".to_string(),
        "notes.md".to_string(),
    ];
    let flags = AppFlags::from_args(args);
    assert_eq!(flags.files.len(), 2);
    assert_eq!(flags.files[0], PathBuf::from("/tmp/doc.txt"));
    assert!(flags.files[1].is_absolute());
    assert!(flags.files[1].ends_with("notes.md"));

    // 10. Empty flags
    let empty_flags = AppFlags::from_args(Vec::<String>::new());
    assert!(empty_flags.files.is_empty());
}

#[test]
fn test_editor_pane_open_file_welcome_tab_reuse_and_non_existent() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use std::path::Path;

    let mut pane = EditorPane::new(PaneId::Left, "Welcome");
    assert_eq!(pane.tabs.len(), 1);
    assert_eq!(pane.active_tab().file_name, "Welcome");
    assert!(pane.active_tab().file_path.is_none());

    // Opening non-existent file in untouched Welcome tab should reuse tab
    let non_existent = Path::new("/tmp/rooney_non_existent_12345.txt");
    let res = pane.open_file(non_existent);
    assert!(res.is_ok());
    assert_eq!(pane.tabs.len(), 1, "Should reuse the initial Welcome tab");
    assert_eq!(pane.active_tab().file_name, "rooney_non_existent_12345.txt");
    assert_eq!(pane.active_tab().file_path.as_deref(), Some(non_existent));
    assert!(pane.active_tab().buffer.full_text().is_empty());

    // Opening another file should now open a second tab
    let another_file = Path::new("/tmp/rooney_second_file_999.rs");
    let res2 = pane.open_file(another_file);
    assert!(res2.is_ok());
    assert_eq!(pane.tabs.len(), 2, "Should create a second tab now");
    assert_eq!(pane.active_tab_idx, 1);
    assert_eq!(pane.active_tab().file_name, "rooney_second_file_999.rs");

    // Switching back to first file if reopened
    let res3 = pane.open_file(non_existent);
    assert!(res3.is_ok());
    assert_eq!(pane.tabs.len(), 2, "Should not duplicate existing tab");
    assert_eq!(pane.active_tab_idx, 0, "Should switch to existing tab");
}

#[test]
fn test_treesitter_multibyte_and_emoji_highlight_coordinates() {
    use rooney::syntax::highlighter::{Highlighter, SupportedLanguage, TokenType};

    // 1. Rust code with Japanese string literal and subsequent code on the same line
    let code = "let greeting = \"こんにちは世界 🚀\"; let count = 100;";
    let mut highlighter = Highlighter::new(SupportedLanguage::Rust);
    highlighter.update_source(code);

    let spans = highlighter.highlight_line(code, 0);
    assert!(!spans.is_empty(), "Should produce spans for Rust code");

    // Convert code to chars vector to index by character
    let chars: Vec<char> = code.chars().collect();

    // Verify all spans map to valid character boundaries and correct text segments
    for span in &spans {
        assert!(
            span.start_col <= span.end_col,
            "start_col {} must be <= end_col {}",
            span.start_col,
            span.end_col
        );
        assert!(
            span.end_col <= chars.len(),
            "end_col {} exceeds char length {}",
            span.end_col,
            chars.len()
        );
    }

    // Verify first "let" keyword
    let first_let = spans.iter().find(|s| s.token_type == TokenType::Keyword && s.start_col == 0);
    assert!(first_let.is_some(), "First let keyword not found");
    assert_eq!(first_let.unwrap().end_col, 3);
    let seg1: String = chars[first_let.unwrap().start_col..first_let.unwrap().end_col].iter().collect();
    assert_eq!(seg1, "let");

    // Verify string literal containing Japanese and emoji
    let string_span = spans.iter().find(|s| s.token_type == TokenType::StringLit);
    assert!(string_span.is_some(), "String literal span not found");
    let s_span = string_span.unwrap();
    let str_content: String = chars[s_span.start_col..s_span.end_col].iter().collect();
    assert_eq!(str_content, "\"こんにちは世界 🚀\"");

    // Verify the second "let" keyword following the multi-byte string
    let second_let = spans
        .iter()
        .find(|s| s.token_type == TokenType::Keyword && s.start_col > s_span.end_col);
    assert!(
        second_let.is_some(),
        "Second let keyword after Japanese/emoji string not found. Spans: {spans:?}"
    );
    let let2 = second_let.unwrap();
    let seg2: String = chars[let2.start_col..let2.end_col].iter().collect();
    assert_eq!(
        seg2, "let",
        "Second let should accurately match chars, got '{seg2}'"
    );

    // Verify the number literal "100"
    let num_span = spans.iter().find(|s| s.token_type == TokenType::NumberLit);
    assert!(num_span.is_some(), "Number literal 100 not found");
    let n = num_span.unwrap();
    let seg_num: String = chars[n.start_col..n.end_col].iter().collect();
    assert_eq!(seg_num, "100");
}

#[test]
fn test_byte_char_mapper_and_slice_by_char_indices() {
    use rooney::syntax::highlighter::{byte_to_char_idx, ByteCharMapper};
    use rooney::ui::canvas_editor::EditorCanvas;

    let text = "Rust 言語 🦀 と 🚀 絵文字!";
    // Chars:
    // 'R'(0), 'u'(1), 's'(2), 't'(3), ' '(4),
    // '言'(5, bytes 5..8), '語'(6, bytes 8..11), ' '(7, byte 11),
    // '🦀'(8, bytes 12..16), ' '(9, byte 16),
    // 'と'(10, bytes 17..20), ' '(11, byte 20),
    // '🚀'(12, bytes 21..25), ' '(13, byte 25),
    // '絵'(14, bytes 26..29), '文'(15, bytes 29..32), '字'(16, bytes 32..35), '!'(17, byte 35)
    // Total chars = 18.
    let mapper = ByteCharMapper::new(text);
    assert_eq!(mapper.total_chars(), 18);

    // Start of string
    assert_eq!(mapper.byte_to_char(0), 0);
    assert_eq!(byte_to_char_idx(text, 0), 0);

    // '言' start byte (5)
    assert_eq!(mapper.byte_to_char(5), 5);
    assert_eq!(byte_to_char_idx(text, 5), 5);

    // '語' start byte (8)
    assert_eq!(mapper.byte_to_char(8), 6);
    assert_eq!(byte_to_char_idx(text, 8), 6);

    // '🦀' start byte (12)
    assert_eq!(mapper.byte_to_char(12), 8);
    assert_eq!(byte_to_char_idx(text, 12), 8);

    // '🚀' start byte (21)
    assert_eq!(mapper.byte_to_char(21), 12);
    assert_eq!(byte_to_char_idx(text, 21), 12);

    // Beyond text
    assert_eq!(mapper.byte_to_char(100), 18);
    assert_eq!(byte_to_char_idx(text, 100), 18);

    // Slice by char indices verification
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 0, 4), "Rust");
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 5, 7), "言語");
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 8, 9), "🦀");
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 12, 13), "🚀");
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 14, 17), "絵文字");
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 17, 18), "!");
    assert_eq!(EditorCanvas::slice_by_char_indices(text, 20, 25), "");
}

#[test]
fn test_multiline_block_comment_highlighting() {
    use rooney::syntax::highlighter::{Highlighter, SupportedLanguage, TokenType};

    let code = "/* 日本語コメント行１\n   中間行\n   終了行 */\nlet z = 1;";
    let mut highlighter = Highlighter::new(SupportedLanguage::Rust);
    highlighter.update_source(code);

    let lines: Vec<&str> = code.lines().collect();

    // Line 0
    let spans0 = highlighter.highlight_line(lines[0], 0);
    assert!(!spans0.is_empty());
    assert_eq!(spans0[0].token_type, TokenType::Comment);
    assert_eq!(spans0[0].start_col, 0);
    assert_eq!(spans0[0].end_col, lines[0].chars().count());

    // Line 1
    let spans1 = highlighter.highlight_line(lines[1], 1);
    assert!(!spans1.is_empty(), "Middle line of block comment should be highlighted");
    assert_eq!(spans1[0].token_type, TokenType::Comment);
    assert_eq!(spans1[0].start_col, 0);
    assert_eq!(spans1[0].end_col, lines[1].chars().count());

    // Line 2
    let spans2 = highlighter.highlight_line(lines[2], 2);
    assert!(!spans2.is_empty(), "End line of block comment should be highlighted");
    assert_eq!(spans2[0].token_type, TokenType::Comment);

    // Line 3 (code line)
    let spans3 = highlighter.highlight_line(lines[3], 3);
    assert!(!spans3.is_empty());
    let let_span = spans3.iter().find(|s| s.token_type == TokenType::Keyword).unwrap();
    assert_eq!(let_span.start_col, 0);
    assert_eq!(let_span.end_col, 3);
}

#[test]
fn test_cosmic_text_glyph_layout() {
    use cosmic_text::{Attrs, Buffer, Family, FontSystem, Metrics, Shaping};
    use rooney::ui::canvas_editor::EditorCanvas;
    use std::sync::Mutex;

    static TEST_FONT_SYSTEM: Mutex<Option<FontSystem>> = Mutex::new(None);
    {
        let mut guard = TEST_FONT_SYSTEM.lock().unwrap();
        if guard.is_none() {
            *guard = Some(FontSystem::new());
        }
    }

    let text = "「正確に見せる」――この2方向で";
    let font_size = 14.0;
    let font_name = "JetBrainsMono Nerd Font";

    // 1. Verify that '―' (U+2015 Horizontal Bar) advances by full-width 14.0px
    let dash_adv = EditorCanvas::char_advance_with_font('―', font_size, font_name);
    assert!(
        (dash_adv - 14.0).abs() < 1e-4,
        "Horizontal bar '―' (U+2015) must advance 14.0px, got {dash_adv}"
    );

    // 2. Verify individual glyph advances match cosmic-text's actual line layout
    let mut cumulative_x = 0.0;
    for c in text.chars() {
        let adv = EditorCanvas::char_advance_with_font(c, font_size, font_name);
        cumulative_x += adv;
    }

    let mut guard = TEST_FONT_SYSTEM.lock().unwrap();
    let fs = guard.as_mut().unwrap();
    let metrics = Metrics::new(font_size, 21.0);
    let mut buffer = Buffer::new(fs, metrics);
    buffer.set_text(
        fs,
        text,
        Attrs::new().family(Family::Name(font_name)),
        Shaping::Advanced,
    );
    buffer.shape_until_scroll(fs, false);

    let full_run = buffer.layout_runs().next().unwrap();
    let full_w = full_run.line_w;

    assert!(
        (cumulative_x - full_w).abs() < 0.01,
        "Individual glyph advances sum ({cumulative_x:.2}) must match full line width ({full_w:.2})!"
    );
}

#[test]
fn test_mixed_script_coordinates_and_roundtrip() {
    use cosmic::iced::{Point, Rectangle, Size};
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::theme::EditorTheme;
    use rooney::ui::canvas_editor::EditorCanvas;

    let test_cases = [
        // 1. Japanese punctuation & East Asian Ambiguous characters (quotes, dashes, ellipsis)
        ("「正確に見せる」――この2方向で……（検証中）！", "Japanese Punctuation & Dashes"),
        // 2. Emojis with ZWJ and surrogate pairs
        ("🦀 Rust 🚀 and 👨‍💻 Hacker", "Emojis & ZWJ Sequence"),
        // 3. Combining characters (e + acute accent, kana dakuten)
        ("Cafe\u{0301} & か\u{3099}", "Combining Characters"),
        // 4. Tabs, spaces, CJK and ASCII mixed
        ("\tfn main() {\t// こんにちは世界！", "Tabs + CJK + ASCII"),
    ];

    let theme = EditorTheme::default();
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(1600.0, 800.0));
    let font_size = 14.0;
    let font_name = "JetBrainsMono Nerd Font";

    for (text, label) in test_cases {
        let mut pane = EditorPane::new(PaneId::Left, "MixedScriptTest");
        pane.buffer = TextBuffer::new(text);
        let chars: Vec<char> = text.lines().next().unwrap_or("").chars().collect();

        // 1. Strictly monotonic increase of character positions
        let mut x_positions = Vec::new();
        for col in 0..=chars.len() {
            pane.buffer.cursor = (0, col);
            let canvas = EditorCanvas::new(&pane, &theme, true, font_name, font_size);
            let pos = canvas.cursor_screen_pos(bounds).expect("cursor_screen_pos must be Some");
            x_positions.push(pos.x);
        }

        for i in 0..x_positions.len() - 1 {
            assert!(
                x_positions[i + 1] > x_positions[i],
                "[{label}] Cursor X position must strictly increase at col {i}: prev={}, next={}",
                x_positions[i],
                x_positions[i + 1]
            );
        }

        // 2. Round-trip: pos_to_char_coords at grapheme cluster boundary returns valid boundary
        let boundaries = pane.buffer.line_grapheme_boundaries(0);
        for &start_col in &boundaries[..boundaries.len() - 1] {
            let next_boundary = pane.buffer.next_grapheme_boundary(0, start_col);
            pane.buffer.cursor = (0, start_col);
            let canvas = EditorCanvas::new(&pane, &theme, true, font_name, font_size);
            let cursor_pos = canvas.cursor_screen_pos(bounds).unwrap();

            // Clicking right on the grapheme start should map to start_col
            let (hit_line, hit_col) = canvas.pos_to_char_coords(cursor_pos, bounds);
            assert_eq!(hit_line, 0, "[{label}] Hit line mismatch at col {start_col}");
            assert_eq!(hit_col, start_col, "[{label}] Round-trip hit col mismatch at col {start_col}");

            pane.buffer.cursor = (0, next_boundary);
            let canvas_next = EditorCanvas::new(&pane, &theme, true, font_name, font_size);
            let next_pos = canvas_next.cursor_screen_pos(bounds).unwrap();
            pane.buffer.cursor = (0, start_col);
            let canvas = EditorCanvas::new(&pane, &theme, true, font_name, font_size);
            let cursor_pos = canvas.cursor_screen_pos(bounds).unwrap();
            let cluster_w = next_pos.x - cursor_pos.x;

            // Clicking in the left half of grapheme cluster should still map to start_col
            let left_half_pt = Point::new(cursor_pos.x + cluster_w * 0.25, cursor_pos.y);
            let (_, left_col) = canvas.pos_to_char_coords(left_half_pt, bounds);
            assert_eq!(left_col, start_col, "[{label}] Left half click mismatch at col {start_col}");

            // Clicking in the right half of grapheme cluster should advance to next_boundary
            let right_half_pt = Point::new(cursor_pos.x + cluster_w * 0.75, cursor_pos.y);
            let (_, right_col) = canvas.pos_to_char_coords(right_half_pt, bounds);
            assert_eq!(right_col, next_boundary, "[{label}] Right half click mismatch at col {start_col}");
        }
    }
}

#[test]
fn test_grapheme_cluster_navigation_and_deletion() {
    // 1. Combining acute accent: Cafe\u{0301}
    // "Cafe\u{0301}" has chars: ['C', 'a', 'f', 'e', '\u{0301}']
    // Char indices: C=0, a=1, f=2, e=3, \u{0301}=4, end=5
    // Grapheme boundaries: [0, 1, 2, 3, 5]
    let mut buffer = TextBuffer::new("Cafe\u{0301}");
    assert_eq!(buffer.line_grapheme_boundaries(0), vec![0, 1, 2, 3, 5]);

    // Move right from start
    buffer.cursor = (0, 0);
    buffer.move_right(false);
    assert_eq!(buffer.cursor, (0, 1));
    buffer.move_right(false);
    assert_eq!(buffer.cursor, (0, 2));
    buffer.move_right(false);
    assert_eq!(buffer.cursor, (0, 3));
    buffer.move_right(false);
    // Skips 4 (\u{0301}) and jumps directly to 5!
    assert_eq!(buffer.cursor, (0, 5));

    // Move left from end
    buffer.move_left(false);
    // Skips 4 and jumps directly to 3!
    assert_eq!(buffer.cursor, (0, 3));

    // Backspace from end (5) deletes both 'e' and '\u{0301}' atomically
    buffer.cursor = (0, 5);
    buffer.delete_backspace();
    assert_eq!(buffer.full_text(), "Caf");
    assert_eq!(buffer.cursor, (0, 3));

    // Delete forward on grapheme cluster
    let mut buffer2 = TextBuffer::new("Cafe\u{0301}!");
    buffer2.cursor = (0, 3); // before 'e\u{0301}'
    buffer2.delete_forward();
    assert_eq!(buffer2.full_text(), "Caf!");
    assert_eq!(buffer2.cursor, (0, 3));

    // 2. Emoji with ZWJ: 👨‍💻 (U+1F468, U+200D, U+1F4BB)
    let mut buffer_emoji = TextBuffer::new("A👨‍💻B");
    // Chars: 'A' (0), '👨' (1), '\u{200D}' (2), '💻' (3), 'B' (4)
    // Grapheme boundaries: [0, 1, 4, 5]
    assert_eq!(buffer_emoji.line_grapheme_boundaries(0), vec![0, 1, 4, 5]);

    buffer_emoji.cursor = (0, 1);
    buffer_emoji.move_right(false);
    assert_eq!(buffer_emoji.cursor, (0, 4)); // jumps whole ZWJ cluster
    buffer_emoji.move_left(false);
    assert_eq!(buffer_emoji.cursor, (0, 1));

    // Backspace at 4 deletes entire 👨‍💻
    buffer_emoji.cursor = (0, 4);
    buffer_emoji.delete_backspace();
    assert_eq!(buffer_emoji.full_text(), "AB");
    assert_eq!(buffer_emoji.cursor, (0, 1));
}

#[test]
fn test_state_consistency_undo_redo_and_cache() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::syntax::highlighter::{HighlightSpan, Highlighter, SupportedLanguage};

    let initial = "fn calculate(x: i32) -> i32 {\n    x * 2\n}\n";
    let mut pane = EditorPane::new(PaneId::Left, "calc.rs");
    pane.buffer = TextBuffer::new(initial);
    pane.file_path = Some(std::path::PathBuf::from("calc.rs"));

    let mut highlighter = Highlighter::new(SupportedLanguage::Rust);
    let text = pane.buffer.full_text();
    highlighter.update_source(&text);

    let hl_line0: Vec<HighlightSpan> = highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert!(!hl_line0.is_empty());
    assert_eq!(highlighter.cached_line_count(), 1);

    // Edit 1: insert comment on line 0
    pane.buffer.cursor = (0, 0);
    pane.buffer.insert_str("// compute double\n");
    assert_eq!(pane.buffer.line_count(), 5);
    if let Some(edit) = pane.buffer.last_edit.take() {
        highlighter.apply_edit(&edit);
    }
    highlighter.update_source(&pane.buffer.full_text());
    assert_eq!(highlighter.cached_line_count(), 0);

    let hl_new0 = highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert!(!hl_new0.is_empty());

    // Undo edit 1
    pane.buffer.undo();
    assert_eq!(pane.buffer.full_text(), initial);
    assert_eq!(pane.buffer.line_count(), 4);
    highlighter.update_source(&pane.buffer.full_text());
    let hl_undo0 = highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert_eq!(hl_undo0, hl_line0);

    // Redo edit 1
    pane.buffer.redo();
    assert_eq!(pane.buffer.full_text(), format!("// compute double\n{}", initial));
    assert_eq!(pane.buffer.line_count(), 5);
    highlighter.update_source(&pane.buffer.full_text());
    let hl_redo0 = highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert_eq!(hl_redo0, hl_new0);
}

#[test]
fn test_glyph_cache_invalidation_lifecycle() {
    use rooney::ui::canvas_editor::{clear_glyph_cache, measure_glyph_advance};

    clear_glyph_cache();

    // Measure at font size 14.0
    let adv_14 = measure_glyph_advance('M', 14.0, "JetBrainsMono Nerd Font");
    assert!(adv_14 > 0.0);

    // Repeated call hits cache
    let adv_14_cached = measure_glyph_advance('M', 14.0, "JetBrainsMono Nerd Font");
    assert_eq!(adv_14, adv_14_cached);

    // Invalidate glyph cache (e.g. font size change / DPI change)
    clear_glyph_cache();

    // Measure at font size 28.0
    let adv_28 = measure_glyph_advance('M', 28.0, "JetBrainsMono Nerd Font");
    assert!(adv_28 > adv_14 * 1.5, "Adv at 28pt ({adv_28}) should be roughly double 14pt ({adv_14})");

    clear_glyph_cache();
}

#[test]
fn test_debounced_parse_state_transition() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::syntax::highlighter::SupportedLanguage;

    let initial = "let value: usize = 12345;\n";
    let mut pane = EditorPane::new(PaneId::Left, "large_bench.rs");
    pane.buffer = TextBuffer::new(initial);
    pane.file_path = Some(std::path::PathBuf::from("large_bench.rs"));
    pane.highlighter = rooney::syntax::highlighter::Highlighter::new(SupportedLanguage::Rust);

    // Initial highlight pass
    let hl_init = pane.highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert!(!hl_init.is_empty());
    assert!(!pane.needs_highlight_parse);

    // Simulate large file (> 2MB) content change
    // Trigger on_content_changed with > 2MB buffer
    let large_line = "let a = 1;\n".repeat(200_000); // ~2.2 MB
    pane.buffer = TextBuffer::new(&large_line);
    pane.on_content_changed();
    assert!(pane.needs_highlight_parse, "Needs highlight parse should be true for >2MB buffer");

    // During debounce, line highlighting still works (via fallback lexical or cached tree)
    let hl_debounced = pane.highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert!(!hl_debounced.is_empty());

    // Flush debounced parse
    pane.flush_highlight_parse();
    assert!(!pane.needs_highlight_parse, "Needs highlight parse must be false after flush");

    // Highlight after flush works reliably
    let hl_flushed = pane.highlighter.highlight_line(&pane.buffer.line_text(0).unwrap(), 0);
    assert!(!hl_flushed.is_empty());
}

#[test]
fn test_wrapped_subrows_cumulative_y_and_hit_test() {
    use cosmic::iced::{Point, Rectangle, Size};
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::theme::EditorTheme;
    use rooney::ui::canvas_editor::EditorCanvas;

    // Line 0: ~75 characters ASCII. With char_width = 8.4 (font_size 14.0), width is ~630px.
    // Line 1: short line.
    // Line 2: short line.
    let text = "012345678901234567890123456789012345678901234567890123456789012345678912345\nLine 1\nLine 2\n";
    let mut pane = EditorPane::new(PaneId::Left, "wrap_test.rs");
    pane.buffer = TextBuffer::new(text);

    let theme = EditorTheme::default();
    let font_size = 14.0f32;
    let line_height = (font_size * 1.5).round(); // 21.0
    let canvas = EditorCanvas::new(&pane, &theme, true, "monospace", font_size);

    let gutter = canvas.gutter_width();
    // Configure avail_width = 252.0 (exactly 30 ASCII chars of 8.4px width per subrow)
    let avail_width = 252.0;
    let bounds_width = avail_width + gutter + 24.0;
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(bounds_width, 800.0));

    let wrap_model = canvas.build_wrap_model(avail_width);

    // Line 0 (75 chars) wraps into 3 subrows (30 chars, 30 chars, 15 chars)
    assert_eq!(wrap_model.subrow_count(0), 3, "Line 0 must wrap into 3 subrows");
    assert_eq!(wrap_model.subrow_count(1), 1, "Line 1 must be 1 row");
    assert_eq!(wrap_model.subrow_count(2), 1, "Line 2 must be 1 row");

    // Line 0 starts at visual row 0
    assert_eq!(wrap_model.line_to_visual_row(0), 0);
    // Line 1 must start at visual row 3 (placed strictly below subrow 2 of Line 0)
    assert_eq!(wrap_model.line_to_visual_row(1), 3);
    // Line 2 must start at visual row 4
    assert_eq!(wrap_model.line_to_visual_row(2), 4);

    // Total visual rows: 3 + 1 + 1 + 1 (trailing empty line) = 6
    let total_vrows = wrap_model.total_visual_rows(pane.buffer.line_count());
    assert_eq!(total_vrows, 6);

    // Verify visual_row_to_line reverse lookup
    assert_eq!(wrap_model.visual_row_to_line(0), (0, 0));
    assert_eq!(wrap_model.visual_row_to_line(1), (0, 1));
    assert_eq!(wrap_model.visual_row_to_line(2), (0, 2));
    assert_eq!(wrap_model.visual_row_to_line(3), (1, 0));
    assert_eq!(wrap_model.visual_row_to_line(4), (2, 0));

    // Verify total_content_height_with_width reflects total visual rows
    let expected_height = (total_vrows as f32) * line_height;
    assert_eq!(canvas.total_content_height_with_width(bounds_width), expected_height);

    // Verify build_viewport_visual_rows non-overlapping Y coordinates
    let visual_rows = canvas.build_viewport_visual_rows(bounds_width, 0.0, 800.0);
    assert_eq!(visual_rows[0].line_idx, 0);
    assert_eq!(visual_rows[0].y, 0.0);

    assert_eq!(visual_rows[1].line_idx, 0);
    assert_eq!(visual_rows[1].y, 1.0 * line_height);

    assert_eq!(visual_rows[2].line_idx, 0);
    assert_eq!(visual_rows[2].y, 2.0 * line_height);

    // Line 1 MUST be at 3 * line_height (strictly non-overlapping!)
    assert_eq!(visual_rows[3].line_idx, 1);
    assert_eq!(visual_rows[3].y, 3.0 * line_height);

    // Line 2 MUST be at 4 * line_height
    assert_eq!(visual_rows[4].line_idx, 2);
    assert_eq!(visual_rows[4].y, 4.0 * line_height);

    // Verify pos_to_char_coords Y hit-testing on wrapped subrows
    // Clicking on Line 0 subrow 0 (y = 0.5 * line_height)
    let (line, col) = canvas.pos_to_char_coords(Point::new(gutter + 15.0, 0.5 * line_height), bounds);
    assert_eq!(line, 0);
    assert!(col < 30);

    // Clicking on Line 0 subrow 1 (y = 1.5 * line_height)
    let (line, col) = canvas.pos_to_char_coords(Point::new(gutter + 15.0, 1.5 * line_height), bounds);
    assert_eq!(line, 0);
    assert!((30..60).contains(&col), "Must map to subrow 1 (col in 30..60), got {col}");

    // Clicking on Line 0 subrow 2 (y = 2.5 * line_height)
    let (line, col) = canvas.pos_to_char_coords(Point::new(gutter + 15.0, 2.5 * line_height), bounds);
    assert_eq!(line, 0);
    assert!(col >= 60, "Must map to subrow 2 (col >= 60), got {col}");

    // Clicking on Line 1 (y = 3.5 * line_height)
    let (line, _) = canvas.pos_to_char_coords(Point::new(gutter + 15.0, 3.5 * line_height), bounds);
    assert_eq!(line, 1, "Must map to Line 1");

    // Clicking on Line 2 (y = 4.5 * line_height)
    let (line, _) = canvas.pos_to_char_coords(Point::new(gutter + 15.0, 4.5 * line_height), bounds);
    assert_eq!(line, 2, "Must map to Line 2");
}

#[test]
fn test_large_file_search_debouncing() {
    use rooney::editor::pane::{EditorPane, PaneId};

    // Construct a ~3MB buffer (exceeding 2MB threshold)
    let line = "let counter_var = 42;\n";
    let text = line.repeat(140_000); // ~3.08 MB
    let mut pane = EditorPane::new(PaneId::Left, "large_search.rs");
    pane.buffer = TextBuffer::new(&text);

    // Set search query
    pane.search_query = Some("counter_var".to_string());
    pane.update_search("counter_var");
    assert!(!pane.search_matches.is_empty());
    assert!(!pane.needs_search_update);

    // Simulate typing in large file
    pane.buffer.cursor = (0, 0);
    let t_type = std::time::Instant::now();
    pane.buffer.insert_char('x');
    pane.on_content_changed();
    let typing_latency_us = t_type.elapsed().as_secs_f64() * 1_000_000.0;

    // Typing must be instantaneous (< 1000 µs) and NOT perform full 3MB scan
    assert!(
        typing_latency_us < 1000.0,
        "Typing with search query on 3MB file took {:.2} µs (must be < 1000 µs)",
        typing_latency_us
    );
    assert!(pane.needs_search_update, "needs_search_update must be set to true for > 2MB buffer");

    // Simulate idle debounce flush
    pane.flush_pending_search();
    assert!(!pane.needs_search_update, "needs_search_update must be reset to false after flush");
    assert!(!pane.search_matches.is_empty());
}

#[test]
fn test_async_search_generation_and_stale_discard() {
    use rooney::editor::pane::{run_search_on_rope, EditorPane, PaneId};

    let text = "alpha beta gamma\nhello world\nalpha delta\n";
    let mut pane = EditorPane::new(PaneId::Left, "async_search_test.rs");
    pane.buffer = TextBuffer::new(text);

    // Test pure helper run_search_on_rope
    let matches = run_search_on_rope(&pane.buffer.rope, "alpha");
    assert_eq!(matches.len(), 2);
    assert_eq!(matches[0], (0, 0, 5));
    assert_eq!(matches[1], (2, 0, 5));

    // Test start_search generation handling
    let (gen1, is_large) = pane.start_search("alpha");
    assert_eq!(gen1, 1);
    assert!(!is_large, "Small file is marked !is_large");
    assert_eq!(pane.search_matches.len(), 2);

    // User types faster: start search for "beta"
    let (gen2, _) = pane.start_search("beta");
    assert_eq!(gen2, 2);

    // User types faster again: start search for "gamma"
    let (gen3, _) = pane.start_search("gamma");
    assert_eq!(gen3, 3);

    // Suppose an older background search worker for "beta" (gen 2) finishes now with 1 match:
    let stale_matches = vec![(0, 6, 10)];
    let applied = pane.apply_search_results(gen2, stale_matches);
    assert!(!applied, "Stale search generation (2 vs 3) must be discarded");

    // Current background worker for "gamma" (gen 3) finishes with 1 match:
    let current_matches = vec![(0, 11, 16)];
    let applied = pane.apply_search_results(gen3, current_matches.clone());
    assert!(applied, "Matching search generation (3) must be applied");
    assert_eq!(pane.search_matches, current_matches);

    // Clearing query (empty string) resets matches and search query
    let (_, _) = pane.start_search("");
    assert!(pane.search_matches.is_empty());
    assert!(pane.search_query.is_none());
}

#[test]
fn test_large_markdown_async_preview_sync() {
    use rooney::config::MarkdownSpec;
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::markdown::MarkdownDocument;

    let mut pane = EditorPane::new(PaneId::Left, "large_doc.md");
    pane.is_markdown_preview = true;
    pane.highlighter = rooney::syntax::highlighter::Highlighter::new(rooney::syntax::SupportedLanguage::Markdown);

    let md_text = "# Header Title\n\n- [x] Completed Task\n- [ ] Pending Task\n\n| Col A | Col B |\n|---|---|\n| 1 | 2 |\n";
    pane.buffer = TextBuffer::new(md_text);

    let edit_time = std::time::Instant::now();
    pane.last_edit_time = edit_time;

    // Simulate background worker parsing markdown
    let doc = MarkdownDocument::parse(md_text, MarkdownSpec::Gfm);
    assert_eq!(doc.blocks.len(), 4); // Heading, 2 Task ListItems, Table

    // Simulate Message::MarkdownParseCompleted handler application
    let tab = pane.active_tab_mut();
    let applied = tab.apply_markdown_doc(tab.markdown_generation, doc);
    assert!(applied);
    tab.needs_highlight_parse = false;

    assert!(pane.markdown_doc.is_some(), "Markdown document must be updated on parse complete");
    let blocks = &pane.markdown_doc.as_ref().unwrap().blocks;
    assert_eq!(blocks.len(), 4);
}

#[test]
fn test_wrap_cache_invalidation_on_font_change() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::theme::EditorTheme;
    use rooney::ui::canvas_editor::EditorCanvas;

    // Line of 60 ASCII characters
    let text = "012345678901234567890123456789012345678901234567890123456789\n";
    let mut pane = EditorPane::new(PaneId::Left, "font_wrap.rs");
    pane.buffer = TextBuffer::new(text);

    let theme = EditorTheme::default();
    let avail_width = 300.0f32; // 300px available

    // Font size 10.0: char width = 6.0px -> 60 chars = 360px > 300px -> wraps into 2 rows
    let canvas_small = EditorCanvas::new(&pane, &theme, true, "monospace", 10.0);
    let wrap_small = canvas_small.build_wrap_model(avail_width);
    assert!(wrap_small.subrow_count(0) >= 2);

    // Invalidate wrap cache via pane method
    pane.invalidate_wrap_cache();
    assert!(pane.active_tab().cached_wrap_model.read().unwrap().is_none());

    // Font size 20.0: char width = 12.0px -> 60 chars = 720px > 300px -> wraps into >= 3 rows
    let canvas_large = EditorCanvas::new(&pane, &theme, true, "monospace", 20.0);
    let wrap_large = canvas_large.build_wrap_model(avail_width);
    assert!(wrap_large.subrow_count(0) >= 3, "Larger font size must wrap into more subrows");

    // Also verify composite keying: even without explicit invalidate_wrap_cache,
    // canvas with different font_size misses the cached entry automatically!
    let wrap_small_again = canvas_small.build_wrap_model(avail_width);
    assert_eq!(wrap_small_again.subrow_count(0), wrap_small.subrow_count(0));
}

#[test]
fn test_search_generation_incremented_on_content_edit() {
    use rooney::editor::buffer::TextBuffer;
    use rooney::editor::pane::{EditorPane, PaneId};

    // Construct a buffer > 2MB
    let chunk = "fn target_symbol() -> i32 { 42 }\n";
    let count = (3 * 1024 * 1024 / chunk.len()) + 10;
    let large_text = chunk.repeat(count);

    let mut pane = EditorPane::new(PaneId::Left, "large_search.rs");
    pane.buffer = TextBuffer::new(&large_text);

    // 1. Start search: sets search_query and increments search_generation to gen1
    let (gen1, is_large) = pane.start_search("target_symbol");
    assert!(is_large, "File > 2MB must be marked as large for search");
    assert!(pane.search_query.is_some());
    assert_eq!(pane.search_generation, gen1);

    // Simulate background worker being in flight with gen1...
    // 2. User edits document while worker is in flight
    pane.buffer.insert_str("// User added a comment\n");
    pane.on_content_changed();

    // search_generation must have incremented on content changed!
    assert!(pane.search_generation > gen1, "Editing buffer during search must advance search_generation");
    assert!(pane.needs_search_update, "Large file edit must flag needs_search_update");

    // 3. In-flight worker for gen1 finishes and tries to apply results
    let stale_matches = vec![(0, 3, 16)];
    let tab = pane.active_tab_mut();
    let applied = tab.apply_search_results(gen1, stale_matches);
    assert!(!applied, "apply_search_results must reject stale generation from before the edit");
    assert!(tab.search_matches.is_empty(), "Stale matches must not be saved into tab");

    // 4. Tick debounced search runs for current generation
    let current_gen = tab.search_generation;
    let fresh_matches = vec![(1, 3, 16)];
    let applied_fresh = tab.apply_search_results(current_gen, fresh_matches.clone());
    assert!(applied_fresh, "apply_search_results must accept current generation");
    assert_eq!(tab.search_matches, fresh_matches);
}

#[test]
fn test_large_markdown_toggle_and_spec_change_async() {
    use rooney::config::MarkdownSpec;
    use rooney::editor::buffer::TextBuffer;
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::markdown::MarkdownDocument;

    // Construct a markdown buffer > 2MB
    let md_chunk = "# Section Title\n\n- [x] Item completed\n- [ ] Item pending\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n";
    let count = (3 * 1024 * 1024 / md_chunk.len()) + 10;
    let large_md = md_chunk.repeat(count);

    let mut pane = EditorPane::new(PaneId::Left, "huge_guide.md");
    pane.buffer = TextBuffer::new(&large_md);
    pane.is_markdown_preview = true;

    // In large file, refresh_markdown must not synchronously parse
    pane.markdown_doc = None;
    pane.refresh_markdown();
    assert!(pane.markdown_doc.is_none(), "Large markdown files must not parse synchronously in refresh_markdown");

    // Background worker parses markdown asynchronously and produces MarkdownDocument
    let edit_time = pane.last_edit_time;
    let doc = MarkdownDocument::parse(&large_md[..1000], MarkdownSpec::Gfm);

    // Apply Message::MarkdownParseCompleted
    let tab = pane.active_tab_mut();
    if tab.last_edit_time == edit_time {
        tab.markdown_doc = Some(doc);
    }
    assert!(pane.markdown_doc.is_some(), "Markdown doc must be updated when background worker completes");
}

#[test]
fn test_markdown_spec_change_generation_race() {
    use rooney::config::MarkdownSpec;
    use rooney::editor::buffer::TextBuffer;
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::markdown::MarkdownDocument;

    let chunk = "# Title\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n";
    let large_md = chunk.repeat(3 * 1024 * 1024 / chunk.len() + 10);

    let mut pane = EditorPane::new(PaneId::Left, "race.md");
    pane.buffer = TextBuffer::new(&large_md);
    pane.is_markdown_preview = true;
    pane.markdown_spec = MarkdownSpec::Gfm;

    // Generation 0
    let gen_gfm = pane.markdown_generation;

    // Simulate worker 1 starting in background with gen_gfm...
    // User switches Spec to CommonMark before worker 1 returns:
    pane.set_markdown_spec(MarkdownSpec::CommonMark);
    let gen_cm = pane.markdown_generation;
    assert!(gen_cm > gen_gfm, "Spec change must advance markdown_generation");

    // Worker 1 (GFM) finishes and attempts to apply results:
    let gfm_doc = MarkdownDocument::parse("| A | B |\n|---|---|\n| 1 | 2 |", MarkdownSpec::Gfm);
    let tab = pane.active_tab_mut();
    let applied_stale = tab.apply_markdown_doc(gen_gfm, gfm_doc);
    assert!(!applied_stale, "Stale generation GFM doc must be discarded");
    assert!(tab.markdown_doc.is_none(), "Markdown doc must not be set by stale worker");

    // Worker 2 (CommonMark) finishes for current generation:
    let cm_doc = MarkdownDocument::parse("| A | B |\n|---|---|\n| 1 | 2 |", MarkdownSpec::CommonMark);
    let applied_fresh = tab.apply_markdown_doc(gen_cm, cm_doc);
    assert!(applied_fresh, "Matching generation CommonMark doc must be applied");
    assert!(tab.markdown_doc.is_some(), "Markdown doc must be set by current worker");
}

#[test]
fn test_markdown_preview_toggle_race() {
    use rooney::config::MarkdownSpec;
    use rooney::editor::buffer::TextBuffer;
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::markdown::MarkdownDocument;

    let chunk = "# Title\n\nSome paragraph text.\n\n";
    let large_md = chunk.repeat(3 * 1024 * 1024 / chunk.len() + 10);

    let mut pane = EditorPane::new(PaneId::Left, "toggle_race.md");
    pane.buffer = TextBuffer::new(&large_md);

    // Turn preview ON: advances generation to gen1
    pane.is_markdown_preview = true;
    pane.markdown_generation = pane.markdown_generation.wrapping_add(1);
    let gen_on = pane.markdown_generation;

    // While worker is in flight, user turns preview OFF:
    pane.is_markdown_preview = false;
    pane.markdown_generation = pane.markdown_generation.wrapping_add(1);
    pane.markdown_doc = None;

    // Worker finishes with gen_on:
    let doc = MarkdownDocument::parse("# Title\n\nSome paragraph text.", MarkdownSpec::Gfm);
    let tab = pane.active_tab_mut();
    let applied = tab.apply_markdown_doc(gen_on, doc);
    assert!(!applied, "Worker results must be discarded when preview was turned OFF and generation changed");
    assert!(tab.markdown_doc.is_none(), "Markdown doc must remain None");
}

#[test]
fn test_all_tabs_markdown_spec_sync() {
    use rooney::config::MarkdownSpec;
    use rooney::editor::buffer::TextBuffer;
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::markdown::MarkdownDocument;

    let chunk = "# Title\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n";
    let large_md = chunk.repeat(3 * 1024 * 1024 / chunk.len() + 10);

    let mut pane = EditorPane::new(PaneId::Left, "tab0.rs");
    // Tab 0 is active (code)
    assert_eq!(pane.active_tab_idx, 0);

    // Tab 1 is inactive (large Markdown file with preview enabled)
    let tab1_idx = pane.new_tab("tab1.md");
    pane.tabs[tab1_idx].buffer = TextBuffer::new(&large_md);
    pane.tabs[tab1_idx].is_markdown_preview = true;
    pane.tabs[tab1_idx].markdown_spec = MarkdownSpec::Gfm;

    // Switch active tab back to Tab 0
    pane.active_tab_idx = 0;
    assert_eq!(pane.active_tab_idx, 0);

    let tab1_old_gen = pane.tabs[1].markdown_generation;

    // User changes spec to CommonMark
    pane.set_markdown_spec(MarkdownSpec::CommonMark);

    // Verify Tab 1 (inactive) has updated spec and advanced generation
    assert_eq!(pane.tabs[1].markdown_spec, MarkdownSpec::CommonMark);
    assert!(pane.tabs[1].markdown_generation > tab1_old_gen, "Inactive tab markdown_generation must be advanced");

    // Simulate async worker for Tab 1 completing with Tab 1's new generation
    let tab1_new_gen = pane.tabs[1].markdown_generation;
    let cm_doc = MarkdownDocument::parse(&large_md[..500], MarkdownSpec::CommonMark);
    let applied = pane.tabs[1].apply_markdown_doc(tab1_new_gen, cm_doc);
    assert!(applied, "Inactive tab must accept parsed doc for its matching generation");
    assert!(pane.tabs[1].markdown_doc.is_some());
}

#[test]
fn test_treesitter_file_switch_parse_generation_race() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::syntax::SupportedLanguage;

    let dir = std::env::temp_dir();
    let file_rs = dir.join("test_ts_race_a.rs");
    let file_py = dir.join("test_ts_race_b.py");
    std::fs::write(&file_rs, "fn main() { println!(\"Hello\"); }").unwrap();
    std::fs::write(&file_py, "def hello():\n    print(\"Hello\")\n").unwrap();

    let mut pane = EditorPane::new(PaneId::Left, "Untitled");
    pane.active_tab_mut().load_file(&file_rs).unwrap();

    let initial_gen = pane.active_tab().parse_generation;
    assert_eq!(pane.active_tab().highlighter.lang, SupportedLanguage::Rust);

    // Simulate async parse starting for file_rs (generation = initial_gen)
    let mut parser = tree_sitter::Parser::new();
    let rs_ts_lang = SupportedLanguage::Rust.tree_sitter_language().unwrap();
    parser.set_language(&rs_ts_lang).unwrap();
    let tree_rs = parser.parse("fn main() { println!(\"Hello\"); }", None);

    // Before worker completes, user loads file_py into the same tab
    pane.active_tab_mut().load_file(&file_py).unwrap();
    let new_gen = pane.active_tab().parse_generation;
    assert_ne!(initial_gen, new_gen, "load_file must increment parse_generation");
    assert_eq!(pane.active_tab().highlighter.lang, SupportedLanguage::Python);

    // Stale tree from file_rs returns with initial_gen
    let applied = pane.active_tab_mut().apply_highlight_tree(initial_gen, tree_rs);
    assert!(!applied, "Stale AST from old file must be rejected");

    // Fresh tree for file_py with new_gen arrives
    let mut py_parser = tree_sitter::Parser::new();
    let py_ts_lang = SupportedLanguage::Python.tree_sitter_language().unwrap();
    py_parser.set_language(&py_ts_lang).unwrap();
    let tree_py = py_parser.parse("def hello():\n    print(\"Hello\")\n", None);

    let applied_py = pane.active_tab_mut().apply_highlight_tree(new_gen, tree_py);
    assert!(applied_py, "Matching generation AST must be accepted");
    assert!(pane.active_tab().highlighter.has_tree(), "Highlighter must now have the Python tree");

    let _ = std::fs::remove_file(file_rs);
    let _ = std::fs::remove_file(file_py);
}

#[test]
fn test_treesitter_save_as_language_change_race() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::syntax::SupportedLanguage;

    let dir = std::env::temp_dir();
    let file_py = dir.join("test_save_as_race.py");

    let mut pane = EditorPane::new(PaneId::Left, "scratch.txt");
    pane.active_tab_mut().buffer.insert_str("print('hello')\n");
    pane.active_tab_mut().on_content_changed();
    let gen_before_save = pane.active_tab().parse_generation;

    // Simulate background parse dispatched for PlainText / Rust
    let mut parser = tree_sitter::Parser::new();
    let dummy_tree = parser.parse("print('hello')\n", None);

    // User performs save_file_as to python file
    pane.active_tab_mut().save_file_as(&file_py).unwrap();
    let gen_after_save = pane.active_tab().parse_generation;
    assert!(gen_after_save > gen_before_save, "save_file_as must increment parse_generation");
    assert_eq!(pane.active_tab().highlighter.lang, SupportedLanguage::Python);

    // Stale parse returns with gen_before_save
    let applied = pane.active_tab_mut().apply_highlight_tree(gen_before_save, dummy_tree);
    assert!(!applied, "Stale AST before save_as must be rejected");

    // Correct Python parse returns
    let mut py_parser = tree_sitter::Parser::new();
    let py_lang = SupportedLanguage::Python.tree_sitter_language().unwrap();
    py_parser.set_language(&py_lang).unwrap();
    let py_tree = py_parser.parse("print('hello')\n", None);

    let applied_correct = pane.active_tab_mut().apply_highlight_tree(gen_after_save, py_tree);
    assert!(applied_correct, "Current generation AST must be applied");

    let _ = std::fs::remove_file(file_py);
}

#[test]
fn test_treesitter_undo_redo_parse_generation_stale_discard() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::syntax::SupportedLanguage;

    let mut pane = EditorPane::new(PaneId::Left, "code.rs");
    pane.active_tab_mut().highlighter = rooney::syntax::highlighter::Highlighter::new(SupportedLanguage::Rust);
    pane.active_tab_mut().buffer.insert_str("fn foo() {}");
    pane.active_tab_mut().on_content_changed();
    let gen_edit1 = pane.active_tab().parse_generation;

    // Worker started for edit1
    let mut parser = tree_sitter::Parser::new();
    let rs_lang = SupportedLanguage::Rust.tree_sitter_language().unwrap();
    parser.set_language(&rs_lang).unwrap();
    let tree_edit1 = parser.parse("fn foo() {}", None);

    // User undoes the change
    pane.active_tab_mut().undo();
    assert_eq!(pane.active_tab().buffer.full_text(), "");
    let gen_undo = pane.active_tab().parse_generation;
    assert!(gen_undo > gen_edit1, "undo() must increment parse_generation");

    // Worker for edit1 completes late
    let applied_stale = pane.active_tab_mut().apply_highlight_tree(gen_edit1, tree_edit1);
    assert!(!applied_stale, "Stale tree from before undo must be discarded");

    // Worker for undo state completes
    let tree_undo = parser.parse("", None);
    let applied_undo = pane.active_tab_mut().apply_highlight_tree(gen_undo, tree_undo);
    assert!(applied_undo, "AST matching undo generation must be applied");

    // User redoes the change
    pane.active_tab_mut().redo();
    assert_eq!(pane.active_tab().buffer.full_text(), "fn foo() {}");
    let gen_redo = pane.active_tab().parse_generation;
    assert!(gen_redo > gen_undo, "redo() must increment parse_generation");

    // Late tree for undo state should be discarded
    let late_undo_tree = parser.parse("", None);
    let applied_late = pane.active_tab_mut().apply_highlight_tree(gen_undo, late_undo_tree);
    assert!(!applied_late, "Stale tree from before redo must be discarded");
}

#[test]
fn test_treesitter_stale_completion_preserves_is_parsing_async() {
    use rooney::editor::pane::{EditorPane, PaneId};
    use rooney::syntax::SupportedLanguage;

    let mut pane = EditorPane::new(PaneId::Left, "main.rs");
    pane.active_tab_mut().highlighter = rooney::syntax::highlighter::Highlighter::new(SupportedLanguage::Rust);
    pane.active_tab_mut().buffer.insert_str("fn first() {}\n");
    pane.active_tab_mut().on_content_changed();
    let gen_worker_a = pane.active_tab().parse_generation;

    // Simulate Worker A parsing tree for gen_worker_a
    let mut parser = tree_sitter::Parser::new();
    let rs_lang = SupportedLanguage::Rust.tree_sitter_language().unwrap();
    parser.set_language(&rs_lang).unwrap();
    let tree_a = parser.parse("fn first() {}\n", None);

    // User makes an edit while Worker A is running:
    // This increments parse_generation, and Tick would start Worker B setting is_parsing_async = true
    pane.active_tab_mut().buffer.insert_char('x');
    pane.active_tab_mut().on_content_changed();
    let gen_worker_b = pane.active_tab().parse_generation;
    assert!(gen_worker_b > gen_worker_a);

    // Mark async parsing active for Worker B
    pane.active_tab_mut().is_parsing_async = true;

    // Worker A finishes late (stale completion with gen_worker_a)
    let applied_a = pane.active_tab_mut().apply_highlight_tree(gen_worker_a, tree_a);
    assert!(!applied_a, "Stale worker completion must return false");
    assert!(
        pane.active_tab().is_parsing_async,
        "is_parsing_async must remain true when stale completion arrives, so running worker B is not corrupted"
    );
    assert!(
        pane.active_tab().needs_highlight_parse,
        "needs_highlight_parse must remain true on stale completion"
    );

    // Worker B completes with matching generation
    let tree_b = parser.parse("xfn first() {}\n", None);
    let applied_b = pane.active_tab_mut().apply_highlight_tree(gen_worker_b, tree_b);
    assert!(applied_b, "Current generation completion must be applied");
    assert!(
        !pane.active_tab().is_parsing_async,
        "is_parsing_async must be reset to false when current generation completes"
    );
    assert!(
        !pane.active_tab().needs_highlight_parse,
        "needs_highlight_parse must be cleared on successful parse application"
    );
}

#[test]
fn test_nested_list_parent_child_retention() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{InlineSpan, MarkdownBlock, MarkdownDocument};

    // Test 1 — Parent + Child
    let md1 = "- Parent item\n  - Child item";
    let doc1 = MarkdownDocument::parse(md1, MarkdownSpec::Gfm);
    assert_eq!(doc1.blocks.len(), 2, "Test 1: Must contain both parent and child items");
    match &doc1.blocks[0] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 0);
            assert_eq!(spans, &[InlineSpan::Text("Parent item".to_string())]);
        }
        _ => panic!("Expected ListItem for parent"),
    }
    match &doc1.blocks[1] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 1);
            assert_eq!(spans, &[InlineSpan::Text("Child item".to_string())]);
        }
        _ => panic!("Expected ListItem for child"),
    }

    // Test 2 — Deep nesting (3 levels)
    let md2 = "- Parent\n  - Child\n    - Grandchild";
    let doc2 = MarkdownDocument::parse(md2, MarkdownSpec::Gfm);
    assert_eq!(doc2.blocks.len(), 3, "Test 2: All 3 nesting levels must be retained");
    match &doc2.blocks[0] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 0);
            assert_eq!(spans, &[InlineSpan::Text("Parent".to_string())]);
        }
        _ => panic!("Expected Parent ListItem"),
    }
    match &doc2.blocks[1] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 1);
            assert_eq!(spans, &[InlineSpan::Text("Child".to_string())]);
        }
        _ => panic!("Expected Child ListItem"),
    }
    match &doc2.blocks[2] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 2);
            assert_eq!(spans, &[InlineSpan::Text("Grandchild".to_string())]);
        }
        _ => panic!("Expected Grandchild ListItem"),
    }

    // Test 3 — Sibling children
    let md3 = "- Parent\n  - Child A\n  - Child B";
    let doc3 = MarkdownDocument::parse(md3, MarkdownSpec::Gfm);
    assert_eq!(doc3.blocks.len(), 3, "Test 3: Parent and both sibling children must be retained");
    match &doc3.blocks[0] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 0);
            assert_eq!(spans, &[InlineSpan::Text("Parent".to_string())]);
        }
        _ => panic!("Expected Parent ListItem"),
    }
    match &doc3.blocks[1] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 1);
            assert_eq!(spans, &[InlineSpan::Text("Child A".to_string())]);
        }
        _ => panic!("Expected Child A ListItem"),
    }
    match &doc3.blocks[2] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 1);
            assert_eq!(spans, &[InlineSpan::Text("Child B".to_string())]);
        }
        _ => panic!("Expected Child B ListItem"),
    }

    // Test 4 — Parent siblings
    let md4 = "- Parent A\n  - Child\n- Parent B";
    let doc4 = MarkdownDocument::parse(md4, MarkdownSpec::Gfm);
    assert_eq!(doc4.blocks.len(), 3, "Test 4: Parent A, Child, and Parent B must be retained");
    match &doc4.blocks[0] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 0);
            assert_eq!(spans, &[InlineSpan::Text("Parent A".to_string())]);
        }
        _ => panic!("Expected Parent A ListItem"),
    }
    match &doc4.blocks[1] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 1);
            assert_eq!(spans, &[InlineSpan::Text("Child".to_string())]);
        }
        _ => panic!("Expected Child ListItem"),
    }
    match &doc4.blocks[2] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 0);
            assert_eq!(spans, &[InlineSpan::Text("Parent B".to_string())]);
        }
        _ => panic!("Expected Parent B ListItem"),
    }

    // Test 5 — Inline formatting
    let md5 = "- **Parent**\n  - *Child*";
    let doc5 = MarkdownDocument::parse(md5, MarkdownSpec::Gfm);
    assert_eq!(doc5.blocks.len(), 2, "Test 5: Both parent and child must retain inline formatting");
    match &doc5.blocks[0] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 0);
            assert_eq!(spans, &[InlineSpan::Bold("Parent".to_string())]);
        }
        _ => panic!("Expected Bold Parent ListItem"),
    }
    match &doc5.blocks[1] {
        MarkdownBlock::ListItem { depth, spans, .. } => {
            assert_eq!(*depth, 1);
            assert_eq!(spans, &[InlineSpan::Italic("Child".to_string())]);
        }
        _ => panic!("Expected Italic Child ListItem"),
    }

    // Additional check: Complex nested list with siblings at multiple depths
    let md_complex = "- Parent\n  - Child\n    - Grandchild\n  - Another child\n- Second parent";
    let doc_complex = MarkdownDocument::parse(md_complex, MarkdownSpec::Gfm);
    assert_eq!(doc_complex.blocks.len(), 5);
    let structure: Vec<(usize, String)> = doc_complex
        .blocks
        .iter()
        .map(|b| match b {
            MarkdownBlock::ListItem { depth, spans, .. } => (*depth, rooney::markdown::renderer::spans_plain_text(spans)),
            _ => panic!("Expected ListItem"),
        })
        .collect();
    assert_eq!(
        structure,
        vec![
            (0, "Parent".to_string()),
            (1, "Child".to_string()),
            (2, "Grandchild".to_string()),
            (1, "Another child".to_string()),
            (0, "Second parent".to_string()),
        ]
    );

    // Additional check: Task list with nesting
    let md_tasks = "- [ ] Parent task\n  - [x] Child task";
    let doc_tasks = MarkdownDocument::parse(md_tasks, MarkdownSpec::Gfm);
    assert_eq!(doc_tasks.blocks.len(), 2);
    match &doc_tasks.blocks[0] {
        MarkdownBlock::ListItem { depth, task_status, spans } => {
            assert_eq!(*depth, 0);
            assert_eq!(*task_status, Some(false));
            assert_eq!(spans, &[InlineSpan::Text("Parent task".to_string())]);
        }
        _ => panic!("Expected ListItem"),
    }
    match &doc_tasks.blocks[1] {
        MarkdownBlock::ListItem { depth, task_status, spans } => {
            assert_eq!(*depth, 1);
            assert_eq!(*task_status, Some(true));
            assert_eq!(spans, &[InlineSpan::Text("Child task".to_string())]);
        }
        _ => panic!("Expected ListItem"),
    }
}

#[test]
fn test_gfm_nested_inline_formatting() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{InlineSpan, InlineStyle, MarkdownBlock, MarkdownDocument};

    // 1. Single styles
    let md_single = "*italic* **bold** ~~strike~~ `code`";
    let doc_single = MarkdownDocument::parse(md_single, MarkdownSpec::Gfm);
    assert_eq!(doc_single.blocks.len(), 1);
    match &doc_single.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Italic(t) if t == "italic")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Bold(t) if t == "bold")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Strikethrough(t) if t == "strike")));
            assert!(spans.iter().any(|s| matches!(s, InlineSpan::Code(t) if t == "code")));
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 2. ***bold italic***
    let md_bi = "***bold italic***";
    let doc_bi = MarkdownDocument::parse(md_bi, MarkdownSpec::Gfm);
    match &doc_bi.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans.len(), 1);
            match &spans[0] {
                InlineSpan::Styled { text, style } => {
                    assert_eq!(text, "bold italic");
                    assert!(style.bold);
                    assert!(style.italic);
                    assert!(!style.strike);
                }
                _ => panic!("Expected Styled span with bold and italic"),
            }
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 3. ~~**bold strike**~~
    let md_bs = "~~**bold strike**~~";
    let doc_bs = MarkdownDocument::parse(md_bs, MarkdownSpec::Gfm);
    match &doc_bs.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans.len(), 1);
            match &spans[0] {
                InlineSpan::Styled { text, style } => {
                    assert_eq!(text, "bold strike");
                    assert!(style.bold);
                    assert!(style.strike);
                    assert!(!style.italic);
                }
                _ => panic!("Expected Styled span with bold and strike"),
            }
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 4. **bold *italic***
    let md_b_i = "**bold *italic***";
    let doc_b_i = MarkdownDocument::parse(md_b_i, MarkdownSpec::Gfm);
    match &doc_b_i.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0], InlineSpan::Bold("bold ".to_string()));
            assert_eq!(
                spans[1],
                InlineSpan::Styled {
                    text: "italic".to_string(),
                    style: InlineStyle {
                        bold: true,
                        italic: true,
                        strike: false,
                    }
                }
            );
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 5. **bold ~~strike~~**
    let md_b_s = "**bold ~~strike~~**";
    let doc_b_s = MarkdownDocument::parse(md_b_s, MarkdownSpec::Gfm);
    match &doc_b_s.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans.len(), 2);
            assert_eq!(spans[0], InlineSpan::Bold("bold ".to_string()));
            assert_eq!(
                spans[1],
                InlineSpan::Styled {
                    text: "strike".to_string(),
                    style: InlineStyle {
                        bold: true,
                        italic: false,
                        strike: true,
                    }
                }
            );
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 6. [**bold link**](https://example.com)
    let md_link = "[**bold link**](https://example.com)";
    let doc_link = MarkdownDocument::parse(md_link, MarkdownSpec::Gfm);
    match &doc_link.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans.len(), 1);
            assert_eq!(
                spans[0],
                InlineSpan::Link {
                    text: "bold link".to_string(),
                    url: "https://example.com".to_string(),
                }
            );
        }
        _ => panic!("Expected Paragraph block"),
    }
}

#[test]
fn test_gfm_links_and_autolinks() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{InlineSpan, MarkdownBlock, MarkdownDocument};

    // Standard Link
    let md1 = "[Rooney](https://example.com)";
    let doc1 = MarkdownDocument::parse(md1, MarkdownSpec::Gfm);
    match &doc1.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(
                spans[0],
                InlineSpan::Link {
                    text: "Rooney".to_string(),
                    url: "https://example.com".to_string(),
                }
            );
        }
        _ => panic!("Expected Paragraph block"),
    }

    // CommonMark Angle Bracket Autolink
    let md2 = "<https://example.com>";
    let doc2 = MarkdownDocument::parse(md2, MarkdownSpec::Gfm);
    match &doc2.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(
                spans[0],
                InlineSpan::Link {
                    text: "https://example.com".to_string(),
                    url: "https://example.com".to_string(),
                }
            );
        }
        _ => panic!("Expected Paragraph block"),
    }

    // GFM Extended Bare Autolink
    let md3 = "Visit https://example.com for info.";
    let doc3 = MarkdownDocument::parse(md3, MarkdownSpec::Gfm);
    match &doc3.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans[0], InlineSpan::Text("Visit ".to_string()));
            assert_eq!(
                spans[1],
                InlineSpan::Link {
                    text: "https://example.com".to_string(),
                    url: "https://example.com".to_string(),
                }
            );
            assert_eq!(spans[2], InlineSpan::Text(" for info.".to_string()));
        }
        _ => panic!("Expected Paragraph block"),
    }
}

#[test]
fn test_gfm_comprehensive_representative_suite() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{ColumnAlignment, InlineSpan, MarkdownBlock, MarkdownDocument};

    // 1. Nested list
    let md_nested = "- one\n  - two\n    - three";
    let doc_nested = MarkdownDocument::parse(md_nested, MarkdownSpec::Gfm);
    assert_eq!(doc_nested.blocks.len(), 3);
    for (i, expected_depth) in [0, 1, 2].iter().enumerate() {
        match &doc_nested.blocks[i] {
            MarkdownBlock::ListItem { depth, .. } => assert_eq!(depth, expected_depth),
            _ => panic!("Expected ListItem"),
        }
    }

    // 2. List + formatting
    let md_list_fmt = "- **bold**\n- *italic*\n- ~~strike~~\n- `code`\n- [link](https://example.com)";
    let doc_list_fmt = MarkdownDocument::parse(md_list_fmt, MarkdownSpec::Gfm);
    assert_eq!(doc_list_fmt.blocks.len(), 5);
    match &doc_list_fmt.blocks[0] {
        MarkdownBlock::ListItem { spans, .. } => {
            assert_eq!(spans, &[InlineSpan::Bold("bold".to_string())]);
        }
        _ => panic!("Expected ListItem"),
    }
    match &doc_list_fmt.blocks[4] {
        MarkdownBlock::ListItem { spans, .. } => {
            assert_eq!(
                spans,
                &[InlineSpan::Link {
                    text: "link".to_string(),
                    url: "https://example.com".to_string(),
                }]
            );
        }
        _ => panic!("Expected ListItem"),
    }

    // 3. List + CodeBlock
    let md_list_code = "- Example:\n\n  ```rust\n  fn main() {}\n  ```\n\n- Next item";
    let doc_list_code = MarkdownDocument::parse(md_list_code, MarkdownSpec::Gfm);
    assert!(doc_list_code.blocks.len() >= 3);
    assert!(matches!(&doc_list_code.blocks[0], MarkdownBlock::ListItem { .. }));
    assert!(matches!(&doc_list_code.blocks[1], MarkdownBlock::CodeBlock { lang, .. } if lang == "rust"));
    assert!(matches!(&doc_list_code.blocks[2], MarkdownBlock::ListItem { .. }));

    // 4. Blockquote with nested list and inline formatting
    let md_bq = "> **bold quote** and *italic*\n>\n> - nested\n> - list";
    let doc_bq = MarkdownDocument::parse(md_bq, MarkdownSpec::Gfm);
    let has_bq = doc_bq.blocks.iter().any(|b| match b {
        MarkdownBlock::BlockQuote(spans) => {
            spans.iter().any(|s| matches!(s, InlineSpan::Bold(t) if t == "bold quote"))
                && spans.iter().any(|s| matches!(s, InlineSpan::Italic(t) if t == "italic"))
        }
        _ => false,
    });
    let has_list_in_bq = doc_bq.blocks.iter().any(|b| matches!(b, MarkdownBlock::ListItem { .. }));
    assert!(has_bq, "Should parse blockquote with inline formatting");
    assert!(has_list_in_bq, "Should retain list items inside blockquote");

    // 5. GFM Table with alignments and escaped pipe
    let md_table = "| Name | Value |\n|:-----|------:|\n| **A** | `100` |\n| *B* | ~~200~~ |\n| foo \\| bar | baz |";
    let doc_table = MarkdownDocument::parse(md_table, MarkdownSpec::Gfm);
    let table_block = doc_table.blocks.iter().find(|b| matches!(b, MarkdownBlock::Table(_)));
    assert!(table_block.is_some(), "Must parse GFM table");
    if let Some(MarkdownBlock::Table(tb)) = table_block {
        assert_eq!(tb.alignments, vec![ColumnAlignment::Left, ColumnAlignment::Right]);
        assert_eq!(tb.headers.len(), 2);
        assert_eq!(tb.rows.len(), 3);
        // Escaped pipe row check
        let escaped_pipe_cell = &tb.rows[2][0];
        let plain = rooney::markdown::renderer::spans_plain_text(escaped_pipe_cell);
        assert!(plain.contains('|'), "Escaped pipe must be preserved inside cell");
    }

    // 6. GFM Table with Japanese / mixed-script
    let md_table_ja = "| 項目名 | 数値 |\n|:---|---:|\n| ルーニー Editor | 144 FPS |";
    let doc_table_ja = MarkdownDocument::parse(md_table_ja, MarkdownSpec::Gfm);
    if let Some(MarkdownBlock::Table(tb)) = doc_table_ja.blocks.iter().find(|b| matches!(b, MarkdownBlock::Table(_))) {
        let plain_h = rooney::markdown::renderer::spans_plain_text(&tb.headers[0]);
        assert_eq!(plain_h, "項目名");
        let plain_v = rooney::markdown::renderer::spans_plain_text(&tb.rows[0][0]);
        assert_eq!(plain_v, "ルーニー Editor");
    }

    // 7. Task list + inline formatting
    let md_task = "- [ ] **Important**\n- [x] ~~Finished~~";
    let doc_task = MarkdownDocument::parse(md_task, MarkdownSpec::Gfm);
    assert_eq!(doc_task.blocks.len(), 2);
    match &doc_task.blocks[0] {
        MarkdownBlock::ListItem { task_status, spans, .. } => {
            assert_eq!(*task_status, Some(false));
            assert_eq!(spans, &[InlineSpan::Bold("Important".to_string())]);
        }
        _ => panic!("Expected ListItem"),
    }
    match &doc_task.blocks[1] {
        MarkdownBlock::ListItem { task_status, spans, .. } => {
            assert_eq!(*task_status, Some(true));
            assert_eq!(spans, &[InlineSpan::Strikethrough("Finished".to_string())]);
        }
        _ => panic!("Expected ListItem"),
    }

    // 8. Multiple Footnotes with formatting
    let md_fn = "Text[^1] and second[^2].\n\n[^1]: **First** footnote.\n[^2]: Second footnote with [link](https://example.com).";
    let doc_fn = MarkdownDocument::parse(md_fn, MarkdownSpec::Gfm);
    let footnotes: Vec<_> = doc_fn.blocks.iter().filter(|b| matches!(b, MarkdownBlock::Footnote { .. })).collect();
    assert_eq!(footnotes.len(), 2);
    if let MarkdownBlock::Footnote { label, spans } = &footnotes[0] {
        assert_eq!(label, "1");
        assert!(spans.iter().any(|s| matches!(s, InlineSpan::Bold(t) if t == "First")));
    }
    if let MarkdownBlock::Footnote { label, spans } = &footnotes[1] {
        assert_eq!(label, "2");
        assert!(spans.iter().any(|s| matches!(s, InlineSpan::Link { .. })));
    }

    // 9. Image fallback (no network access)
    let md_img = "![Rooney Preview](https://example.com/rooney.png)";
    let doc_img = MarkdownDocument::parse(md_img, MarkdownSpec::Gfm);
    match &doc_img.blocks[0] {
        MarkdownBlock::Paragraph(spans) => {
            assert_eq!(spans.len(), 1);
            assert_eq!(
                spans[0],
                InlineSpan::ImageFallback {
                    alt: "Rooney Preview".to_string(),
                    url: "https://example.com/rooney.png".to_string(),
                }
            );
        }
        _ => panic!("Expected Paragraph block"),
    }

    // 10. HTML safe fallback (no webview, safe text)
    let md_html = "<div>Hello</div>\n<img src=\"https://example.com/a.png\" alt=\"Logo\">";
    let doc_html = MarkdownDocument::parse(md_html, MarkdownSpec::Gfm);
    let full_text: String = doc_html.blocks.iter().map(|b| b.plain_text()).collect::<Vec<_>>().join(" ");
    assert!(full_text.contains("Hello"), "Safe HTML text must be retained");
    let has_img = doc_html.blocks.iter().any(|b| match b {
        MarkdownBlock::Paragraph(spans) => spans.iter().any(|s| matches!(s, InlineSpan::ImageFallback { alt, .. } if alt == "Logo")),
        _ => false,
    });
    assert!(has_img, "HTML img tag safely converts to ImageFallback");
}

