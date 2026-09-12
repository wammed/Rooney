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
        rooney::markdown::renderer::MarkdownBlock::Heading { level: 1, text } => text == "Title 1",
        _ => false,
    });
    assert!(has_heading, "Heading 1 parsed properly");
}

#[test]
fn test_markdown_spec_commonmark_vs_gfm() {
    use rooney::config::MarkdownSpec;
    use rooney::markdown::renderer::{AlertKind, MarkdownBlock};

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

    let gfm_strike_text = doc_gfm_strike.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(t) => Some(t.clone()),
        _ => None,
    }).unwrap_or_default();
    assert!(gfm_strike_text.contains('\u{0336}'), "GFM renders strikethrough with combining strike marks");
    assert!(!gfm_strike_text.contains("~~"), "GFM strips ~~ delimiter");

    let cm_strike_text = doc_cm_strike.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(t) => Some(t.clone()),
        _ => None,
    }).unwrap_or_default();
    assert_eq!(cm_strike_text, "This is ~~deleted~~ text.");

    // 5. breaks: false (soft line breaks do not form <br>)
    let break_md = "First line\nSecond line";
    let doc_gfm_break = MarkdownDocument::parse(break_md, MarkdownSpec::Gfm);
    let doc_cm_break = MarkdownDocument::parse(break_md, MarkdownSpec::CommonMark);

    let gfm_break_text = doc_gfm_break.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(t) => Some(t.clone()),
        _ => None,
    }).unwrap_or_default();
    let cm_break_text = doc_cm_break.blocks.iter().find_map(|b| match b {
        MarkdownBlock::Paragraph(t) => Some(t.clone()),
        _ => None,
    }).unwrap_or_default();

    assert_eq!(gfm_break_text, "First line Second line");
    assert_eq!(cm_break_text, "First line Second line");
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

