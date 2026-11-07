use rooney::editor::buffer::TextBuffer;
use rooney::editor::pane::{EditorPane, PaneId, ViewMode};
use rooney::markdown::mapping::parse_line_live_preview;

#[test]
fn test_live_preview_single_source_of_truth() {
    let original = "# Title\n\nThis is **bold** and *italic* and `code`.\n\n- [ ] Task 1\n";
    let mut pane = EditorPane::new(PaneId::Left, "test.md");
    pane.buffer = TextBuffer::new(original);
    pane.view_mode = ViewMode::LivePreview;

    assert!(pane.is_markdown());
    assert_eq!(pane.buffer.full_text(), original);

    // Switching modes does not alter buffer content
    pane.view_mode = ViewMode::Source;
    assert_eq!(pane.buffer.full_text(), original);

    pane.view_mode = ViewMode::Reading;
    assert_eq!(pane.buffer.full_text(), original);
}

#[test]
fn test_live_preview_cursor_expansion_and_collapse() {
    let line = "## 見出し **太字** テスト";

    // 1. When cursor is ON this line (line 0):
    // Syntax markers are unhidden so user sees exact markdown and edits at 1:1 offsets
    let on_line = parse_line_live_preview(line, true);
    assert_eq!(on_line.visual_text, line);
    assert_eq!(on_line.source_map.segments.iter().all(|s| !s.is_hidden), true);

    // 2. When cursor is on another line:
    // Markers are hidden and styles are applied
    let off_line = parse_line_live_preview(line, false);
    assert_eq!(off_line.visual_text, "見出し 太字 テスト");
    assert!(off_line.source_map.segments.iter().any(|s| s.is_hidden));

    // Clicking at '太' in "太字" (visual col 4) maps back to source col 9 (after "## 見出し **")
    let source_col = off_line.source_map.visual_to_source_col(4);
    assert_eq!(source_col, 9);
    assert_eq!(line.chars().nth(source_col), Some('太'));
}

#[test]
fn test_live_preview_inline_code_and_strikethrough_roundtrip() {
    let line = "打ち消し ~~old~~ と コード `cargo test` です";

    let off_line = parse_line_live_preview(line, false);
    assert_eq!(off_line.visual_text, "打ち消し old と コード cargo test です");

    // Click at 'c' in "cargo test" (visual col 13: "打ち消し old と コード ")
    // "打ち消し " (5) + "old " (4) + "と " (2) + "コード " (4) = 15 chars
    let byte_pos = off_line.visual_text.find("cargo").unwrap();
    let code_char_idx = off_line.visual_text[..byte_pos].chars().count();
    let source_idx = off_line.source_map.visual_to_source_col(code_char_idx);
    assert_eq!(line.chars().nth(source_idx), Some('c'));
}

#[test]
fn test_live_preview_editing_preserves_raw_markdown() {
    let mut pane = EditorPane::new(PaneId::Left, "doc.md");
    pane.buffer = TextBuffer::new("**bold**");
    pane.view_mode = ViewMode::LivePreview;
    pane.buffer.cursor = (0, 6); // between 'd' and closing "**"

    // Type 'er'
    pane.buffer.insert_str("er");
    assert_eq!(pane.buffer.full_text(), "**bolder**");

    // Undo reverts back to "**bold**"
    pane.buffer.undo();
    assert_eq!(pane.buffer.full_text(), "**bold**");
}

#[test]
fn test_live_preview_view_mode_transitions() {
    let mut pane = EditorPane::new(PaneId::Left, "notes.md");
    assert_eq!(pane.view_mode, ViewMode::LivePreview);

    // Toggle to Source
    pane.view_mode = match pane.view_mode {
        ViewMode::LivePreview => ViewMode::Source,
        ViewMode::Source => ViewMode::LivePreview,
        ViewMode::Reading => ViewMode::LivePreview,
    };
    assert_eq!(pane.view_mode, ViewMode::Source);

    // Toggle back to LivePreview
    pane.view_mode = match pane.view_mode {
        ViewMode::LivePreview => ViewMode::Source,
        ViewMode::Source => ViewMode::LivePreview,
        ViewMode::Reading => ViewMode::LivePreview,
    };
    assert_eq!(pane.view_mode, ViewMode::LivePreview);
}

#[test]
fn test_live_preview_click_mapping_exact_character_alignment() {
    let line = "これは **太字テキスト** と *斜体テキスト*、~~打消し線~~ です。";
    let off_line = parse_line_live_preview(line, false);

    assert_eq!(
        off_line.visual_text,
        "これは 太字テキスト と 斜体テキスト、打消し線 です。"
    );

    // 1. Verify "太字テキスト" characters map exactly without 1-2 char offsets
    for target_char in ['太', '字', 'テ', 'キ', 'ス', 'ト'] {
        let vis_byte = off_line.visual_text.find(target_char).unwrap();
        let vis_col = off_line.visual_text[..vis_byte].chars().count();
        let source_col = off_line.source_map.visual_to_source_col(vis_col);
        assert_eq!(
            line.chars().nth(source_col),
            Some(target_char),
            "Failed exact alignment for char '{}': expected source char, got {:?}",
            target_char,
            line.chars().nth(source_col)
        );
    }

    // 2. Verify "斜体テキスト" characters map exactly despite earlier hidden "**" markers
    for target_char in ['斜', '体', 'テ', 'キ', 'ス', 'ト'] {
        let vis_byte = off_line.visual_text.find(target_char).unwrap();
        let vis_col = off_line.visual_text[..vis_byte].chars().count();
        let source_col = off_line.source_map.visual_to_source_col(vis_col);
        assert_eq!(
            line.chars().nth(source_col),
            Some(target_char),
            "Failed exact alignment for italic char '{}'",
            target_char
        );
    }

    // 3. Verify trailing text "です。" after multiple hidden markers
    for target_char in ['で', 'す', '。'] {
        let vis_byte = off_line.visual_text.find(target_char).unwrap();
        let vis_col = off_line.visual_text[..vis_byte].chars().count();
        let source_col = off_line.source_map.visual_to_source_col(vis_col);
        assert_eq!(
            line.chars().nth(source_col),
            Some(target_char),
            "Failed exact alignment for trailing char '{}'",
            target_char
        );
    }
}

#[test]
fn test_canvas_pos_to_char_coords_non_cursor_line_live_preview() {
    use cosmic::iced::{Point, Rectangle, Size};
    use rooney::theme::EditorTheme;
    use rooney::ui::canvas_editor::EditorCanvas;

    let content = "First line (cursor here)\nこれは **太字テキスト** です。";
    let mut pane = EditorPane::new(PaneId::Left, "test.md");
    pane.buffer = TextBuffer::new(content);
    pane.view_mode = ViewMode::LivePreview;
    pane.buffer.cursor = (0, 0); // Cursor on line 0, so line 1 is non-cursor Live Preview

    let theme = EditorTheme::default();
    let canvas = EditorCanvas::new(&pane, &theme, true, "monospace", 14.0);
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
    let gutter = canvas.gutter_width();
    let line_height = canvas.line_height;

    // Line 1 Y center coordinate
    let line1_y = line_height * 1.5;

    // Line 1 visually renders as: "これは 太字テキスト です。"
    // "これは " width:
    let prefix_w = canvas.glyph_advance('こ')
        + canvas.glyph_advance('れ')
        + canvas.glyph_advance('は')
        + canvas.glyph_advance(' ');

    // Click on '太': prefix_w + half of '太'
    let tai_x = gutter + 10.0 + prefix_w + canvas.glyph_advance('太') * 0.5;
    let (line, col) = canvas.pos_to_char_coords(Point::new(tai_x, line1_y), bounds);
    assert_eq!(line, 1);
    assert_eq!(col, 6, "Expected col 6 ('太' in 'これは **太字テキスト** です。'), got {col}");
    assert_eq!(pane.buffer.line_text(1).unwrap().chars().nth(col), Some('太'));

    // Click on '字': prefix_w + '太' + half of '字'
    let ji_x = gutter + 10.0 + prefix_w + canvas.glyph_advance('太') + canvas.glyph_advance('字') * 0.5;
    let (line, col) = canvas.pos_to_char_coords(Point::new(ji_x, line1_y), bounds);
    assert_eq!(line, 1);
    assert_eq!(col, 7, "Expected col 7 ('字' in 'これは **太字テキスト** です。'), got {col}");
    assert_eq!(pane.buffer.line_text(1).unwrap().chars().nth(col), Some('字'));

    // Click on 'で': prefix_w + "太字テキスト " + half of 'で'
    let text_w = canvas.glyph_advance('太')
        + canvas.glyph_advance('字')
        + canvas.glyph_advance('テ')
        + canvas.glyph_advance('キ')
        + canvas.glyph_advance('ス')
        + canvas.glyph_advance('ト')
        + canvas.glyph_advance(' ');
    let de_x = gutter + 10.0 + prefix_w + text_w + canvas.glyph_advance('で') * 0.5;
    let (line, col) = canvas.pos_to_char_coords(Point::new(de_x, line1_y), bounds);
    assert_eq!(line, 1);
    assert_eq!(col, 15, "Expected col 15 ('で' in 'これは **太字テキスト** です。'), got {col}");
    assert_eq!(pane.buffer.line_text(1).unwrap().chars().nth(col), Some('で'));
}
