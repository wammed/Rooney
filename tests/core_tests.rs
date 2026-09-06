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

