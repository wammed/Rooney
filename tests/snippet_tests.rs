use rooney::snippet::{parse_snippet_template, SnippetRegistry};

#[test]
fn test_snippet_registry_loads_default_markdown_preset() {
    let registry = SnippetRegistry::new();
    let md = registry.get_language_for_ext("md").expect("Markdown preset should be loaded");
    assert_eq!(md.language.id, "markdown");
    assert_eq!(md.language.name, "Markdown");
    assert!(md.language.extensions.contains(&"md".to_string()));
    assert!(md.language.extensions.contains(&"markdown".to_string()));

    // Check presence of essential categories
    let categories: Vec<&str> = md.snippets.iter().map(|s| s.category.as_str()).collect();
    assert!(categories.contains(&"Formatting"));
    assert!(categories.contains(&"Headings"));
    assert!(categories.contains(&"Structure"));
    assert!(categories.contains(&"Code"));

    // Bold snippet check
    let bold = md.snippets.iter().find(|s| s.label == "Bold").expect("Bold snippet should exist");
    assert_eq!(bold.content, "**${1:bold text}**$0");
}

#[test]
fn test_parse_snippet_template_basic() {
    let template = "**${1:強調テキスト}**$0";
    let parsed = parse_snippet_template(template, None);

    assert_eq!(parsed.text, "**強調テキスト**");
    assert_eq!(parsed.placeholders.len(), 1);
    let p1 = &parsed.placeholders[0];
    assert_eq!(p1.index, 1);
    assert_eq!(p1.default_value, "強調テキスト");
    assert_eq!(p1.start_char, 2);
    assert_eq!(p1.end_char, 8); // "**" is 2 chars, "強調テキスト" is 6 chars -> 2..8
    assert_eq!(parsed.final_caret_char_offset, 10); // After closing "**"
}

#[test]
fn test_parse_snippet_template_with_selected_text() {
    let template = "**${1:強調テキスト}**$0";
    let selected = "選択された文章";
    let parsed = parse_snippet_template(template, Some(selected));

    assert_eq!(parsed.text, "**選択された文章**");
    assert_eq!(parsed.placeholders.len(), 1);
    let p1 = &parsed.placeholders[0];
    assert_eq!(p1.index, 1);
    assert_eq!(p1.default_value, "選択された文章");
    assert_eq!(p1.start_char, 2);
    assert_eq!(p1.end_char, 9); // "**" is 2 chars, "選択された文章" is 7 chars -> 2..9
    assert_eq!(parsed.final_caret_char_offset, 11);
}

#[test]
fn test_parse_snippet_template_escaped_and_math_block() {
    let template = "$$\n${1:E = mc^2}\n$$\n\n$0";
    let parsed = parse_snippet_template(template, None);

    assert_eq!(parsed.text, "$$\nE = mc^2\n$$\n\n");
    assert_eq!(parsed.placeholders.len(), 1);
    let p1 = &parsed.placeholders[0];
    assert_eq!(p1.index, 1);
    assert_eq!(p1.default_value, "E = mc^2");
    assert_eq!(p1.start_char, 3); // "$$\n" is 3 chars
    assert_eq!(p1.end_char, 11); // 3 + 8 chars
    assert_eq!(parsed.final_caret_char_offset, 16);
}

#[test]
fn test_parse_snippet_template_multiple_placeholders_and_mirrors() {
    let template = "fn ${1:func_name}($2) -> ${3:i32} {\n    // log $1\n    $0\n}";
    let parsed = parse_snippet_template(template, None);

    assert_eq!(
        parsed.text,
        "fn func_name() -> i32 {\n    // log func_name\n    \n}"
    );
    // Placeholders should be sorted by index: 1, 1 (mirror), 2, 3
    assert_eq!(parsed.placeholders.len(), 4);
    assert_eq!(parsed.placeholders[0].index, 1);
    assert_eq!(parsed.placeholders[0].default_value, "func_name");
    assert_eq!(parsed.placeholders[1].index, 1); // mirror
    assert!(parsed.placeholders[1].is_mirror);
    assert_eq!(parsed.placeholders[2].index, 2);
    assert_eq!(parsed.placeholders[3].index, 3);
}

#[test]
fn test_insert_snippet_and_placeholder_session_navigation() {
    let mut pane = rooney::editor::pane::EditorPane::new(rooney::editor::pane::PaneId::Left, "doc.md");
    pane.buffer = rooney::editor::buffer::TextBuffer::new("");

    // Insert bold snippet
    pane.insert_snippet("**${1:強調テキスト}**$0");

    assert_eq!(pane.buffer.full_text(), "**強調テキスト**");
    assert!(pane.placeholder_session.is_some());

    // First placeholder should be selected (start=(0, 2), end=(0, 8))
    assert_eq!(pane.buffer.selection_anchor, Some((0, 2)));
    assert_eq!(pane.buffer.cursor, (0, 8));

    // Tab navigates to $0 (end of bold text at col 10) and completes session
    let has_next = pane.tab_next_placeholder();
    assert!(has_next);
    assert!(pane.placeholder_session.is_none());
    assert_eq!(pane.buffer.cursor, (0, 10));
    assert_eq!(pane.buffer.selection_anchor, None);

    // Single undo reverts entire snippet insertion
    pane.buffer.undo();
    assert_eq!(pane.buffer.full_text(), "");

    // Redo restores snippet
    pane.buffer.redo();
    assert_eq!(pane.buffer.full_text(), "**強調テキスト**");
}

#[test]
fn test_insert_snippet_replaces_selection() {
    let mut pane = rooney::editor::pane::EditorPane::new(rooney::editor::pane::PaneId::Left, "test.md");
    pane.buffer = rooney::editor::buffer::TextBuffer::new("前 重要文章 後");

    // Select "重要文章" (col 2..6)
    pane.buffer.selection_anchor = Some((0, 2));
    pane.buffer.cursor = (0, 6);

    // Insert bold snippet
    pane.insert_snippet("**${1:強調テキスト}**$0");

    // Selected text "重要文章" is preserved as initial value of $1
    assert_eq!(pane.buffer.full_text(), "前 **重要文章** 後");

    // "重要文章" is currently selected
    assert_eq!(pane.buffer.selection_anchor, Some((0, 4)));
    assert_eq!(pane.buffer.cursor, (0, 8));

    // 1 undo restores original text and selection
    pane.buffer.undo();
    assert_eq!(pane.buffer.full_text(), "前 重要文章 後");
}

#[test]
fn test_insert_snippet_multi_step_tab_and_shift_tab() {
    let mut pane = rooney::editor::pane::EditorPane::new(rooney::editor::pane::PaneId::Left, "test.rs");
    pane.buffer = rooney::editor::buffer::TextBuffer::new("");

    pane.insert_snippet("fn ${1:name}(${2:arg}: i32) -> ${3:i32} { $0 }");

    assert_eq!(pane.buffer.full_text(), "fn name(arg: i32) -> i32 {  }");
    assert!(pane.placeholder_session.is_some());

    // Step 1: name selected (3..7)
    assert_eq!(pane.buffer.selection_anchor, Some((0, 3)));
    assert_eq!(pane.buffer.cursor, (0, 7));

    // Tab -> Step 2: arg selected (8..11)
    assert!(pane.tab_next_placeholder());
    assert_eq!(pane.buffer.selection_anchor, Some((0, 8)));
    assert_eq!(pane.buffer.cursor, (0, 11));

    // Tab -> Step 3: return type i32 selected (21..24)
    assert!(pane.tab_next_placeholder());
    assert_eq!(pane.buffer.selection_anchor, Some((0, 21)));
    assert_eq!(pane.buffer.cursor, (0, 24));

    // Shift+Tab -> Back to Step 2: arg selected
    assert!(pane.tab_prev_placeholder());
    assert_eq!(pane.buffer.selection_anchor, Some((0, 8)));
    assert_eq!(pane.buffer.cursor, (0, 11));

    // Forward to Step 3
    assert!(pane.tab_next_placeholder());

    // Forward from Step 3 -> $0 (inside braces at 27)
    assert!(pane.tab_next_placeholder());
    assert!(pane.placeholder_session.is_none());
    assert_eq!(pane.buffer.cursor, (0, 27));
}

#[test]
fn test_toggle_task_checkbox_interactive() {
    let mut pane = rooney::editor::pane::EditorPane::new(rooney::editor::pane::PaneId::Left, "tasks.md");
    pane.buffer = rooney::editor::buffer::TextBuffer::new("- [ ] 未完了タスク\n- [x] 完了タスク\n");

    // 1. Toggle line 0 from [ ] to [x]
    let changed = pane.toggle_task_at_line(0);
    assert!(changed);
    assert_eq!(pane.buffer.full_text(), "- [x] 未完了タスク\n- [x] 完了タスク\n");

    // 2. Undo reverts to [ ]
    pane.buffer.undo();
    assert_eq!(pane.buffer.full_text(), "- [ ] 未完了タスク\n- [x] 完了タスク\n");

    // 3. Toggle line 1 from [x] to [ ]
    let changed_line1 = pane.toggle_task_at_line(1);
    assert!(changed_line1);
    assert_eq!(pane.buffer.full_text(), "- [ ] 未完了タスク\n- [ ] 完了タスク\n");
}

#[test]
fn test_backspace_on_selected_placeholder_does_not_crash() {
    use rooney::editor::pane::EditorTab;
    let mut tab = EditorTab::new(1, "test.md");
    // Insert snippet without trailing text
    tab.insert_snippet("> [!NOTE] ${1:Note title}\n> ${2:Note content here.}\n$0");
    assert!(tab.placeholder_session.is_some());
    assert!(tab.buffer.selection_anchor.is_some());

    // User presses Backspace while initial placeholder is selected
    tab.buffer.delete_backspace();
    tab.on_content_changed();

    // Must not crash and placeholder session should adjust cleanly
    assert!(tab.placeholder_session.is_some());
    assert_eq!(tab.buffer.full_text(), "> [!NOTE] \n> Note content here.\n");

    // Tab navigates to placeholder 2 despite placeholder 1 being empty
    let has_next = tab.tab_next_placeholder();
    assert!(has_next);
    assert_eq!(tab.buffer.selection_anchor, Some((1, 2)));
    assert_eq!(tab.buffer.cursor, (1, 20));

    // Another Tab ends session at $0
    tab.tab_next_placeholder();
    assert!(tab.placeholder_session.is_none());
}

#[test]
fn test_snippet_registry_user_directory_override_and_extension() {
    let mut registry = SnippetRegistry::new();
    let temp_dir = std::env::temp_dir().join(format!("rooney_test_snippets_{}", std::process::id()));
    let _ = std::fs::create_dir_all(&temp_dir);

    // Create a custom rust.toml
    let rust_toml = r#"
[language]
id = "rust"
name = "Rust"
extensions = ["rs"]

[[snippets]]
category = "Control Flow"
label = "if let"
description = "if let pattern matching"
content = "if let Some(${1:val}) = ${2:opt} {\n    $0\n}"
"#;
    std::fs::write(temp_dir.join("rust.toml"), rust_toml).unwrap();

    registry.load_from_dir(&temp_dir);

    // Verify rust snippets are registered
    let rust = registry.get_language_for_ext("rs").expect("Rust snippets should be loaded");
    assert_eq!(rust.language.name, "Rust");
    assert!(rust.snippets.iter().any(|s| s.label == "if let"));

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_snippet_registry_loads_all_config_languages() {
    let registry = SnippetRegistry::new();
    let langs = registry.available_languages();
    let lang_ids: Vec<&str> = langs.iter().map(|l| l.id.as_str()).collect();

    assert!(lang_ids.contains(&"markdown"));
    assert!(lang_ids.contains(&"rust"));
    assert!(lang_ids.contains(&"python"));
    assert!(lang_ids.contains(&"bash"));
    assert!(lang_ids.contains(&"typescript"));
    assert!(lang_ids.contains(&"cpp"));
}


