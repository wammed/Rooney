use rooney::markdown::mapping::{
    parse_line_live_preview, DisplaySegment, LineSourceMap, SegmentStyle, SourceRange,
};
use ropey::Rope;

#[test]
fn test_source_range_to_char_range() {
    let rope = Rope::from_str("Hello, 世界! 🦀");
    let r1 = SourceRange::new(0, 7);
    assert_eq!(r1.to_char_range(&rope), 0..7);

    let r2 = SourceRange::new(7, 15);
    assert_eq!(r2.to_char_range(&rope), 7..11);

    let r3 = SourceRange::new(15, 19);
    assert_eq!(r3.to_char_range(&rope), 11..12);
}

#[test]
fn test_line_source_map_hidden_markers() {
    let map = LineSourceMap {
        segments: vec![
            DisplaySegment {
                visual_char_range: 0..0,
                source_char_range: 0..2,
                source_byte_range: 0..2,
                text: "**".to_string(),
                style: SegmentStyle::default(),
                is_syntax_marker: true,
                is_hidden: true,
            },
            DisplaySegment {
                visual_char_range: 0..9,
                source_char_range: 2..11,
                source_byte_range: 2..11,
                text: "bold text".to_string(),
                style: SegmentStyle {
                    bold: true,
                    ..Default::default()
                },
                is_syntax_marker: false,
                is_hidden: false,
            },
            DisplaySegment {
                visual_char_range: 9..9,
                source_char_range: 11..13,
                source_byte_range: 11..13,
                text: "**".to_string(),
                style: SegmentStyle::default(),
                is_syntax_marker: true,
                is_hidden: true,
            },
        ],
    };

    // Visual to Source
    assert_eq!(map.visual_to_source_col(0), 2); // 'b' in "bold"
    assert_eq!(map.visual_to_source_col(4), 6); // ' ' in "bold text"
    assert_eq!(map.visual_to_source_col(8), 10); // 't' in "text"
    assert_eq!(map.visual_to_source_col(9), 13); // end of line -> after trailing "**"

    // Source to Visual
    assert_eq!(map.source_to_visual_col(0), 0); // leading "**" maps to 0
    assert_eq!(map.source_to_visual_col(1), 0);
    assert_eq!(map.source_to_visual_col(2), 0); // 'b'
    assert_eq!(map.source_to_visual_col(6), 4);
    assert_eq!(map.source_to_visual_col(11), 9); // trailing "**" maps to 9
    assert_eq!(map.source_to_visual_col(12), 9);
    assert_eq!(map.source_to_visual_col(13), 9);
}

#[test]
fn test_line_source_map_unhidden_expanded_markers() {
    let map = LineSourceMap {
        segments: vec![
            DisplaySegment {
                visual_char_range: 0..2,
                source_char_range: 0..2,
                source_byte_range: 0..2,
                text: "**".to_string(),
                style: SegmentStyle::default(),
                is_syntax_marker: true,
                is_hidden: false,
            },
            DisplaySegment {
                visual_char_range: 2..11,
                source_char_range: 2..11,
                source_byte_range: 2..11,
                text: "bold text".to_string(),
                style: SegmentStyle {
                    bold: true,
                    ..Default::default()
                },
                is_syntax_marker: false,
                is_hidden: false,
            },
            DisplaySegment {
                visual_char_range: 11..13,
                source_char_range: 11..13,
                source_byte_range: 11..13,
                text: "**".to_string(),
                style: SegmentStyle::default(),
                is_syntax_marker: true,
                is_hidden: false,
            },
        ],
    };

    // 1:1 mapping when unhidden
    for col in 0..=13 {
        assert_eq!(map.visual_to_source_col(col), col);
        assert_eq!(map.source_to_visual_col(col), col);
    }
}

#[test]
fn test_parse_line_live_preview_heading_and_bold() {
    let line = "## 見出し **太字** テスト";

    // 1. Cursor NOT on line: syntax markers hidden
    let rendered = parse_line_live_preview(line, false);
    assert_eq!(rendered.visual_text, "見出し 太字 テスト");
    // "## " (3 chars) hidden, "**" (2 chars) hidden
    assert_eq!(rendered.source_map.visual_to_source_col(0), 3); // '見' is at source col 3
    assert_eq!(rendered.source_map.source_to_visual_col(3), 0);

    // 2. Cursor ON line: syntax markers expanded (1:1 with source)
    let expanded = parse_line_live_preview(line, true);
    assert_eq!(expanded.visual_text, line);
    for col in 0..line.chars().count() {
        assert_eq!(expanded.source_map.visual_to_source_col(col), col);
        assert_eq!(expanded.source_map.source_to_visual_col(col), col);
    }
}

#[test]
fn test_parse_line_live_preview_inline_code_and_link() {
    let line = "コード `let x = 1;` と [リンク](https://example.com) です";

    // Cursor NOT on line
    let rendered = parse_line_live_preview(line, false);
    assert_eq!(
        rendered.visual_text,
        "コード let x = 1; と リンク です"
    );

    // Verify code style on "let x = 1;"
    let code_seg = rendered
        .segments
        .iter()
        .find(|s| s.text == "let x = 1;")
        .expect("Code segment should exist");
    assert!(code_seg.style.code);
    assert!(!code_seg.is_hidden);

    // Verify link style on "リンク"
    let link_seg = rendered
        .segments
        .iter()
        .find(|s| s.text == "リンク")
        .expect("Link segment should exist");
    assert!(link_seg.style.link);
    assert!(!link_seg.is_hidden);

    // Verify URL marker is hidden
    let url_seg = rendered
        .segments
        .iter()
        .find(|s| s.text == "](https://example.com)")
        .expect("URL marker should exist");
    assert!(url_seg.is_hidden);
}


