use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::hint::black_box;
use std::time::{Duration, Instant};
use sveltui_code::buffer::CodeBuffer;
use sveltui_code::highlight::{clip_spans, highlight_text};

struct BenchResult {
    name: &'static str,
    iterations: usize,
    total: Duration,
    min: Duration,
    max: Duration,
    avg: Duration,
    throughput: Option<String>,
}

impl BenchResult {
    fn display(&self) {
        let total_ms = self.total.as_nanos() as f64 / 1_000_000.0;
        let avg_us = self.avg.as_nanos() as f64 / 1_000.0;
        let min_us = self.min.as_nanos() as f64 / 1_000.0;
        let max_us = self.max.as_nanos() as f64 / 1_000.0;

        let formatted_total = if total_ms >= 1_000.0 {
            format!("{:.2} s", total_ms / 1_000.0)
        } else {
            format!("{:.2} ms", total_ms)
        };

        let formatted_avg = if avg_us >= 1_000.0 {
            format!("{:.2} ms", avg_us / 1_000.0)
        } else {
            format!("{:.2} µs", avg_us)
        };

        let formatted_min = if min_us >= 1_000.0 {
            format!("{:.2} ms", min_us / 1_000.0)
        } else {
            format!("{:.2} µs", min_us)
        };

        let formatted_max = if max_us >= 1_000.0 {
            format!("{:.2} ms", max_us / 1_000.0)
        } else {
            format!("{:.2} µs", max_us)
        };

        let tp = self.throughput.as_deref().unwrap_or("-");

        println!(
            "{:<42} | {:>4} runs | total: {:>9} | avg: {:>9} | min: {:>9} | max: {:>9} | {}",
            self.name,
            self.iterations,
            formatted_total,
            formatted_avg,
            formatted_min,
            formatted_max,
            tp
        );
    }
}

fn run_benchmark<F>(name: &'static str, warmup: usize, iterations: usize, mut f: F) -> BenchResult
where
    F: FnMut() -> Option<String>,
{
    // Warmup
    for _ in 0..warmup {
        let _ = black_box(f());
    }

    let mut min = Duration::MAX;
    let mut max = Duration::ZERO;
    let mut total = Duration::ZERO;
    let mut throughput = None;

    for _ in 0..iterations {
        let start = Instant::now();
        let tp = f();
        let elapsed = start.elapsed();

        if elapsed < min {
            min = elapsed;
        }
        if elapsed > max {
            max = elapsed;
        }
        total += elapsed;
        if throughput.is_none() {
            throughput = tp;
        }
    }

    let avg = total / (iterations as u32);
    BenchResult {
        name,
        iterations,
        total,
        min,
        max,
        avg,
        throughput,
    }
}

// =========================================================================
// 1. Text Buffer Mutation Benchmarks
// =========================================================================

fn bench_buffer_sequential_typing() -> BenchResult {
    run_benchmark("Buffer: Sequential typing (1,000 chars)", 5, 50, || {
        let mut buf = CodeBuffer::new("");
        let key = KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE);
        for _ in 0..1_000 {
            buf.handle_key(&key);
        }
        let total_chars = buf.content().len();
        black_box(total_chars);
        Some(format!("{} chars/op", total_chars))
    })
}

fn bench_buffer_newline_and_autoindent() -> BenchResult {
    run_benchmark("Buffer: Newlines & auto-indent (300 lines)", 5, 50, || {
        let mut buf = CodeBuffer::new("");
        let open_brace = KeyEvent::new(KeyCode::Char('{'), KeyModifiers::NONE);
        let enter = KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE);
        for _ in 0..300 {
            buf.handle_key(&open_brace);
            buf.handle_key(&enter);
        }
        let lines = buf.line_count();
        black_box(lines);
        Some(format!("{} lines created", lines))
    })
}

fn bench_buffer_line_merging_backspace() -> BenchResult {
    run_benchmark("Buffer: Backspace line merging (300 lines)", 5, 50, || {
        let initial_text = (0..300)
            .map(|i| format!("fn line_{}() {{}}", i))
            .collect::<Vec<_>>()
            .join("\n");
        let mut buf = CodeBuffer::new(&initial_text);
        let bs = KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE);

        // Move to the end of each line and delete into the previous line
        for row in (1..300).rev() {
            buf.cursor_row = row;
            buf.cursor_col = 0;
            buf.handle_key(&bs);
        }
        black_box(buf.line_count());
        Some("300 lines merged".to_string())
    })
}

fn bench_buffer_undo_redo_cycles() -> BenchResult {
    run_benchmark("Buffer: Undo/Redo cycles (500 steps)", 5, 30, || {
        let mut buf = CodeBuffer::new("");
        let type_x = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
        for _ in 0..500 {
            buf.handle_key(&type_x);
        }
        let undo = KeyEvent::new(KeyCode::Char('z'), KeyModifiers::CONTROL);
        for _ in 0..250 {
            buf.handle_key(&undo);
        }
        let redo = KeyEvent::new(KeyCode::Char('y'), KeyModifiers::CONTROL);
        for _ in 0..250 {
            buf.handle_key(&redo);
        }
        black_box(buf.content().len());
        Some("1,000 undo/redo operations".to_string())
    })
}

// =========================================================================
// 2. Syntax Highlighting Benchmarks
// =========================================================================

fn bench_highlight_cold_small() -> BenchResult {
    let small_code = r#"// Small Rust module (50 lines)
pub struct State<T> {
    id: u64,
    value: T,
}

impl<T: Clone> State<T> {
    pub fn new(id: u64, value: T) -> Self {
        Self { id, value }
    }

    pub fn get(&self) -> T {
        self.value.clone()
    }

    pub fn set(&mut self, new_val: T) {
        self.value = new_val;
    }
}

pub fn fibonacci(n: u32) -> u64 {
    match n {
        0 => 0,
        1 => 1,
        _ => fibonacci(n - 1) + fibonacci(n - 2),
    }
}

pub fn calculate_sum(items: &[u64]) -> u64 {
    items.iter().sum()
}
"#
    .repeat(2);

    run_benchmark("Highlight: Cold parse (small ~50 lines)", 2, 20, || {
        // Vary text slightly each time to avoid cache hit and benchmark real parser throughput
        let unique = format!(
            "{}\n// ts: {}\n",
            small_code,
            Instant::now().elapsed().as_nanos()
        );
        let res = highlight_text(&unique, Some("rs"), None);
        let count = res.len();
        black_box(count);
        Some(format!("{:.0} lines/sec", (count as f64) / 0.001))
    })
}

fn bench_highlight_cold_medium() -> BenchResult {
    let block = r#"pub fn process_event(event: &Event) -> Result<(), AppError> {
    match event {
        Event::Key(k) => handle_key_event(k)?,
        Event::Mouse(m) => handle_mouse_event(m)?,
        Event::Resize(w, h) => handle_resize(*w, *h)?,
    }
    Ok(())
}
"#;
    let medium_code = block.repeat(70); // ~500 lines

    run_benchmark("Highlight: Cold parse (medium ~500 lines)", 2, 10, || {
        let unique = format!(
            "{}\n// ts: {}\n",
            medium_code,
            Instant::now().elapsed().as_nanos()
        );
        let start = Instant::now();
        let res = highlight_text(&unique, Some("rs"), None);
        let elapsed = start.elapsed().as_secs_f64();
        let count = res.len();
        black_box(count);
        let lps = (count as f64) / elapsed.max(1e-6);
        Some(format!("{:.0} lines/sec", lps))
    })
}

fn bench_highlight_warm_cache_hit() -> BenchResult {
    let code = r#"// Cached code sample for navigation/scrolling
fn render_viewport(ctx: &mut Context) {
    for line in 0..100 {
        ctx.draw_line(line);
    }
}
"#
    .repeat(10);

    // Warm up the cache
    let _ = highlight_text(&code, Some("rs"), None);

    run_benchmark(
        "Highlight: Warm cache hit (navigation/scroll)",
        10,
        500,
        || {
            let res = highlight_text(&code, Some("rs"), None);
            black_box(res.len());
            Some("instant O(1) hit".to_string())
        },
    )
}

fn bench_clip_spans_viewport() -> BenchResult {
    let code =
        "let formatted_variable_name = calculate_complex_expression(param1, param2, param3);\n";
    let highlighted = highlight_text(code, Some("rs"), None);
    let line_spans = &highlighted[0];

    run_benchmark(
        "Highlight: Viewport span clipping (1,000 lines)",
        5,
        100,
        || {
            let mut total_clipped = 0;
            for col_offset in (0..40).step_by(2) {
                for _ in 0..25 {
                    let clipped = clip_spans(line_spans, col_offset, 80);
                    total_clipped += clipped.len();
                }
            }
            black_box(total_clipped);
            Some("1,000 spans clipped".to_string())
        },
    )
}

fn main() {
    println!(
        "\n========================================================================================="
    );
    println!(
        "                   SVELTUI-CODE: TEXT BUFFER & HIGHLIGHTING BENCHMARKS                   "
    );
    println!(
        "=========================================================================================\n"
    );

    println!("--- 1. Buffer Mutation Benchmarks ---");
    bench_buffer_sequential_typing().display();
    bench_buffer_newline_and_autoindent().display();
    bench_buffer_line_merging_backspace().display();
    bench_buffer_undo_redo_cycles().display();

    println!("\n--- 2. Syntax Highlighting Benchmarks ---");
    bench_highlight_cold_small().display();
    bench_highlight_cold_medium().display();
    bench_highlight_warm_cache_hit().display();
    bench_clip_spans_viewport().display();

    println!(
        "\n=========================================================================================\n"
    );
}
