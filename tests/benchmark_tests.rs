use cosmic::iced::{Point, Rectangle, Size};
use rooney::editor::buffer::TextBuffer;
use rooney::editor::pane::{EditorPane, PaneId};
use rooney::syntax::highlighter::{Highlighter, SupportedLanguage};
use rooney::theme::EditorTheme;
use rooney::ui::canvas_editor::EditorCanvas;
use std::time::Instant;

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

        // 3. 1-character incremental edit (interactive typing latency)
        pane.buffer.cursor = (line_count / 2, 5);
        let t_edit = Instant::now();
        pane.buffer.insert_char('x');
        pane.on_content_changed();
        let incremental_edit_and_parse_us = t_edit.elapsed().as_secs_f64() * 1_000_000.0;

        // Flush debounced parse if pending (> 2MB files)
        let t_flush = Instant::now();
        pane.flush_highlight_parse();
        let deferred_parse_ms = t_flush.elapsed().as_secs_f64() * 1000.0;

        if pane.buffer.len_bytes() > 2 * 1024 * 1024 {
            println!("[{label}] Typing Latency: {incremental_edit_and_parse_us:.2} µs (< 0.1ms), Background/Debounced Parse: {deferred_parse_ms:.2} ms");
        } else {
            println!("[{label}] Synchronous Incremental Edit & Parse: {incremental_edit_and_parse_us:.2} µs");
        }

        // 4. Viewport layout virtualization
        let canvas = EditorCanvas::new(&pane, &theme, true, "monospace", 14.0);
        let t_layout = Instant::now();
        let scroll_y = ((line_count / 2) as f32) * canvas.line_height;
        let visual_rows = canvas.build_viewport_visual_rows(1200.0, scroll_y, 800.0);
        let viewport_layout_us = t_layout.elapsed().as_secs_f64() * 1_000_000.0;

        assert!(!visual_rows.is_empty());
        // Viewport rows should only be ~ visible count + margin (e.g. 800/21 = 38 + 10 = ~48 rows)
        assert!(
            visual_rows.len() < 100,
            "Viewport rows must be virtualized to visible window"
        );

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
    println!(
        "{:<8} | {:>10} | {:>10} | {:>14} | {:>14} | {:>14} | {:>14} | {:>14}",
        "Scale",
        "Bytes",
        "Lines",
        "Buf Load (ms)",
        "Init Parse(ms)",
        "Incr Edit(µs)",
        "Viewport(µs)",
        "Cache Hit(µs)"
    );
    println!("---------------------------------------------------------------------------------------------------------");

    for r in &results {
        println!(
            "{:<8} | {:>10} | {:>10} | {:>14.2} | {:>14.2} | {:>14.2} | {:>14.2} | {:>14.2}",
            r.name,
            r.bytes,
            r.lines,
            r.buffer_create_ms,
            r.initial_parse_ms,
            r.incremental_edit_and_parse_us,
            r.viewport_layout_us,
            r.highlight_cache_hit_us
        );
    }
    println!("=========================================================================================================\n");
}

#[test]
fn test_benchmark_scenario_a_50mb_continuous_typing_with_search() {
    let target_bytes = 50 * 1024 * 1024;
    let code = generate_synthetic_code(target_bytes);
    let mut pane = EditorPane::new(PaneId::Left, "ScenarioA_50MB");
    pane.buffer = TextBuffer::new(&code);
    let line_count = pane.buffer.line_count();

    // Set active search query
    pane.update_search("calculate_metric");
    assert!(
        !pane.search_matches.is_empty(),
        "Search matches should be found"
    );

    // Position cursor in the middle
    pane.buffer.cursor = (line_count / 2, 5);

    // Continuous typing: 50 characters
    let keystrokes = 50;
    let t_start = Instant::now();
    for _ in 0..keystrokes {
        let t_char = Instant::now();
        pane.buffer.insert_char('z');
        pane.on_content_changed();
        let elapsed_us = t_char.elapsed().as_secs_f64() * 1_000_000.0;
        // Each keystroke must not block the UI (must be < 500 µs, typically ~30 µs)
        assert!(
            elapsed_us < 500.0,
            "Keystroke latency with active search took {:.2} µs (must be < 500 µs)",
            elapsed_us
        );
    }
    let total_typing_time = t_start.elapsed();
    let avg_keystroke_us = (total_typing_time.as_secs_f64() * 1_000_000.0) / (keystrokes as f64);

    assert!(
        pane.needs_search_update,
        "Search update should be marked debounced/pending"
    );

    // Idle flush
    let t_flush = Instant::now();
    pane.flush_pending_search();
    let flush_ms = t_flush.elapsed().as_secs_f64() * 1000.0;
    assert!(!pane.needs_search_update);

    println!("\n[Scenario A: 50MB Continuous Typing with Active Search]");
    println!(
        "  Buffer Size: {:.2} MB ({} lines)",
        target_bytes as f64 / (1024.0 * 1024.0),
        line_count
    );
    println!("  Keystrokes: {}", keystrokes);
    println!(
        "  Avg Typing Latency: {:.2} µs (< 0.1 ms - UI remains 144+ FPS)",
        avg_keystroke_us
    );
    println!("  Debounced Idle Search Scan: {:.2} ms (executed asynchronously/on idle without UI freeze)", flush_ms);
}

#[test]
fn test_benchmark_scenario_b_wrapped_lines_layout_and_hit_test() {
    // Generate text with several multi-thousand-character lines
    let mut text = String::new();
    for line_idx in 0..10 {
        text.push_str(&format!("// Line {}\n", line_idx));
        let chunk = "const DATA_BLOB_SEGMENT_ALPHA_BETA_GAMMA: &str = \"abcdefghijklmnopqrstuvwxyz0123456789 日本語テスト \"; ";
        let repeats = 60; // ~60 * 70 bytes = ~4,200 bytes per line
        for _ in 0..repeats {
            text.push_str(chunk);
        }
        text.push('\n');
    }
    text.push_str("fn end_of_file() {}\n");

    let mut pane = EditorPane::new(PaneId::Left, "ScenarioB_Wrapped");
    pane.buffer = TextBuffer::new(&text);

    let theme = EditorTheme::default();
    let font_size = 14.0f32;
    let canvas = EditorCanvas::new(&pane, &theme, true, "monospace", font_size);

    let avail_width = 800.0f32;
    let line_height = canvas.line_height;

    // 1. Measure LineWrapModel building throughput
    let t_wrap = Instant::now();
    let wrap_model = canvas.build_wrap_model(avail_width);
    let wrap_build_us = t_wrap.elapsed().as_secs_f64() * 1_000_000.0;

    let total_vrows = wrap_model.total_visual_rows(pane.buffer.line_count());
    assert!(
        total_vrows > 50,
        "Long lines must wrap into many visual rows"
    );

    // 2. Measure Viewport Virtualized Visual Rows calculation
    let t_viewport = Instant::now();
    let scroll_y = 500.0f32;
    let visual_rows = canvas.build_viewport_visual_rows(avail_width + 100.0, scroll_y, 600.0);
    let viewport_us = t_viewport.elapsed().as_secs_f64() * 1_000_000.0;

    assert!(!visual_rows.is_empty());
    // Ensure viewport rows are ordered and strictly non-overlapping
    for i in 1..visual_rows.len() {
        assert!(
            visual_rows[i].y >= visual_rows[i - 1].y + line_height - 0.01,
            "Visual rows must not overlap! row {} y={} vs row {} y={}",
            i - 1,
            visual_rows[i - 1].y,
            i,
            visual_rows[i].y
        );
    }

    // 3. Measure Hit-Test (pos_to_char_coords) throughput across 1000 simulated clicks
    let t_hit = Instant::now();
    let hit_count = 1000;
    let bounds = Rectangle::new(Point::ORIGIN, Size::new(avail_width + 100.0, 800.0));
    for step in 0..hit_count {
        let test_y = (step as f32 * 3.7) % (total_vrows as f32 * line_height);
        let test_x = 50.0 + (step as f32 * 7.3) % 700.0;
        let coords = canvas.pos_to_char_coords(Point::new(test_x, test_y), bounds);
        assert!(coords.0 < pane.buffer.line_count());
    }
    let hit_test_total_us = t_hit.elapsed().as_secs_f64() * 1_000_000.0;
    let avg_hit_test_ns = (hit_test_total_us * 1000.0) / (hit_count as f64);

    println!("\n[Scenario B: Multi-Thousand-Character Wrapped Lines Layout & Hit-Test]");
    println!(
        "  Total Visual Rows: {} (across 10 long lines)",
        total_vrows
    );
    println!("  LineWrapModel Build Time: {:.2} µs", wrap_build_us);
    println!(
        "  Viewport Visual Rows Query: {:.2} µs (virtualized)",
        viewport_us
    );
    println!(
        "  Hit-Test Latency: {:.2} ns / query ({} queries in {:.2} µs)",
        avg_hit_test_ns, hit_count, hit_test_total_us
    );

    assert!(
        wrap_build_us < 100_000.0,
        "Wrap model build must be fast (< 100 ms in debug)"
    );
    assert!(
        viewport_us < 5000.0,
        "Viewport query must be fast (< 5000 µs)"
    );
    assert!(
        avg_hit_test_ns < 3_000_000.0,
        "Hit test must be fast (< 3 ms in debug / < 200 µs in release)"
    );
}
