//! One `DataGrid` frame (build, layout, render) at very different row
//! counts. Only the rows on screen are built, so the time should barely
//! move from a thousand rows to a million.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::rc::Rc;
use sveltui::element::{HitMap, render_node_interactive, reset_focus_counter};
use sveltui::layout::Rect;
use sveltui::prelude::*;
use sveltui::terminal::Buffer;
use sveltui_components::*;

#[derive(Clone)]
struct Row {
    id: u64,
    name: String,
    score: f64,
}

fn rows(count: usize) -> Rc<Vec<Row>> {
    Rc::new(
        (0..count as u64)
            .map(|id| Row {
                id,
                name: format!("row {id}"),
                score: (id * 7919 % 1000) as f64 / 10.0,
            })
            .collect(),
    )
}

fn columns() -> Vec<GridColumn<Row>> {
    vec![
        GridColumn::new("Id", |r: &Row| r.id.to_string()).width(length(10)),
        GridColumn::new("Name", |r: &Row| r.name.clone()).width(fr(1.0)),
        GridColumn::new("Score", |r: &Row| format!("{:.1}", r.score))
            .width(length(8))
            .align(CellAlign::End)
            .sort_by(|a, b| a.score.total_cmp(&b.score)),
    ]
}

fn frame(data: &Rc<Vec<Row>>, sorted: bool) {
    reset_focus_counter();
    begin_frame();
    let node = view! {
        <DataGrid rows={GridRows::Shared(data.clone())} columns={columns()} height=40
            default_sort={sorted.then_some(SortState { column: 2, direction: SortDirection::Ascending })} />
    };
    end_frame();
    let mut buffer = Buffer::new(120, 40);
    let mut hits = HitMap::new();
    render_node_interactive(&node, Rect::new(0, 0, 120, 40), &mut buffer, &mut hits);
    black_box(buffer);
}

fn bench_data_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("data_grid_frame");
    for count in [1_000, 100_000, 1_000_000] {
        let data = rows(count);
        for sorted in [false, true] {
            let name = if sorted { "sorted" } else { "unsorted" };
            // Components keep their state from frame to frame, as in an app,
            // so only this first frame sorts.
            frame(&data, sorted);
            group.bench_with_input(BenchmarkId::new(name, count), &count, |b, _| {
                b.iter(|| frame(&data, sorted));
            });
        }
    }
    group.finish();
}

criterion_group!(benches, bench_data_grid);
criterion_main!(benches);
