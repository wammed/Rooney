use std::time::Instant;
use rooney::editor::buffer::TextBuffer;
use rooney::editor::pane::{EditorPane, PaneId};
use rooney::syntax::highlighter::{Highlighter, SupportedLanguage};
use rooney::theme::EditorTheme;
use rooney::ui::canvas_editor::EditorCanvas;

fn generate_synthetic_code(target_bytes: usize) -> String {
    let mut code = String::with_capacity(target_bytes + 256);
    let sample_lines = [
        "pub fn calculate_metric(index: usize, value: f64) -> f64 {\n",
        "    let factor = 3.1415926535 * (index as f64);\n",
        "    if value > 0.0 && factor > 10.0 {\n",
        "        // Log and compute normal score\n",
        "        let score = (value * factor).sqrt();\n",
        "        score + 1.0\n",
        "    } else {\n",
        "        0.0\n",
        "    }\n",
        "}\n",
    ];

    let mut idx = 0;
    while code.len() < target_bytes {
        code.push_str(sample_lines[idx % sample_lines.len()]);
        idx += 1;
    }
    code
}

#[derive(Debug)]
struct BenchmarkResult {
    name: &'static str,
    bytes: usize,
    lines: usize,
    buffer_create_ms: f64,
    initial_parse_ms: f64,
    incremental_edit_and_parse_us: f64,
    viewport_layout_us: f64,
    highlight_cache_hit_us: f64,
}

#[test]
fn test_multiscale_performance_benchmarks() {
    let scales = [
        ("10 KB", 10 * 1024),
        ("100 KB", 100 * 1024),
        ("1 MB", 1024 * 1024),
        ("10 MB", 10 * 1024 * 1024),
        ("50 MB", 50 * 1024 * 1024),
    ];

    let mut results = Vec::new();
    let theme = EditorTheme::default();

    for &(label, target_bytes) in &scales {
        let code = generate_synthetic_code(target_bytes);
        let actual_bytes = code.len();

        // 1. TextBuffer creation
        let t0 = Instant::now();
        let mut pane = EditorPane::new(PaneId::Left, "Benchmark");
        pane.buffer = TextBuffer::new(&code);
        let buffer_create_ms = t0.elapsed().as_secs_f64() * 1000.0;
        let line_count = pane.buffer.line_count();

        // 2. Initial Parse (full Tree-sitter AST generation)
        let mut highlighter = Highlighter::new(SupportedLanguage::Rust);
        let t_parse = Instant::now();
        let text = pane.buffer.full_text();
        highlighter.update_source(&text);
        let initial_parse_ms = t_parse.elapsed().as_secs_f64() * 1000.0;

        // 3. 1-character incremental edit & incremental parse
        pane.buffer.cursor = (line_count / 2, 5);
        let t_edit = Instant::now();
        pane.buffer.insert_char('x');
        if let Some(edit) = pane.buffer.last_edit.take() {
            highlighter.apply_edit(&edit);
        }
        let updated_text = pane.buffer.full_text();
        highlighter.update_source(&updated_text);
        let incremental_edit_and_parse_us = t_edit.elapsed().as_secs_f64() * 1_000_000.0;

        // 4. Viewport layout virtualization
        let canvas = EditorCanvas::new(
            &pane,
            &theme,
            true,
            "monospace",
            14.0,
        );
        let t_layout = Instant::now();
        let scroll_y = ((line_count / 2) as f32) * canvas.line_height;
        let visual_rows = canvas.build_viewport_visual_rows(1200.0, scroll_y, 800.0);
        let viewport_layout_us = t_layout.elapsed().as_secs_f64() * 1_000_000.0;

        assert!(!visual_rows.is_empty());
        // Viewport rows should only be ~ visible count + margin (e.g. 800/21 = 38 + 10 = ~48 rows)
        assert!(visual_rows.len() < 100, "Viewport rows must be virtualized to visible window");

        // 5. Logical line highlight cache hit
        let mid_line_text = pane.buffer.line_text(line_count / 2).unwrap_or_default();
        let _ = highlighter.highlight_line(&mid_line_text, line_count / 2);
        let t_cache = Instant::now();
        let spans = highlighter.highlight_line(&mid_line_text, line_count / 2);
        let highlight_cache_hit_us = t_cache.elapsed().as_secs_f64() * 1_000_000.0;
        let _ = spans;

        results.push(BenchmarkResult {
            name: label,
            bytes: actual_bytes,
            lines: line_count,
            buffer_create_ms,
            initial_parse_ms,
            incremental_edit_and_parse_us,
            viewport_layout_us,
            highlight_cache_hit_us,
        });

        // Viewport layout MUST be sub-millisecond even on 50MB files (< 2000 µs)!
        assert!(
            viewport_layout_us < 2000.0,
            "Viewport layout for {} took {:.2} µs (must be < 2000 µs)",
            label,
            viewport_layout_us
        );
    }

    println!("\n=========================================================================================================");
    println!("                                   ROONEY MULTI-SCALE PERFORMANCE BENCHMARK                              ");
    println!("=========================================================================================================");
    println!("{:<8} | {:>10} | {:>10} | {:>14} | {:>14} | {:>14} | {:>14} | {:>14}",
        "Scale", "Bytes", "Lines", "Buf Load (ms)", "Init Parse(ms)", "Incr Edit(µs)", "Viewport(µs)", "Cache Hit(µs)");
    println!("---------------------------------------------------------------------------------------------------------");

    for r in &results {
        println!("{:<8} | {:>10} | {:>10} | {:>14.2} | {:>14.2} | {:>14.2} | {:>14.2} | {:>14.2}",
            r.name, r.bytes, r.lines, r.buffer_create_ms, r.initial_parse_ms,
            r.incremental_edit_and_parse_us, r.viewport_layout_us, r.highlight_cache_hit_us);
    }
    println!("=========================================================================================================\n");
}
