use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use sveltui::element::taffy_bridge::LayoutEngine;
use sveltui::element::{IntoNode, TextElement, VNode, View, ViewProps};
use sveltui::layout::{FlexDirection, Margin, Padding, Size};

fn create_view_tree(n: usize) -> VNode {
    let mut children = Vec::with_capacity(n);
    for i in 0..n {
        let child = View(
            ViewProps::builder()
                .width(Size::fixed(10))
                .height(Size::fixed(1))
                .margin(Margin::all(1))
                .flex_grow(if i % 2 == 0 { 1.0 } else { 0.0 })
                .child(VNode::Text(TextElement::new(format!("Item {}", i))))
                .build(),
        )
        .into_node();
        children.push(child);
    }

    View(
        ViewProps::builder()
            .width(Size::fixed(120))
            .height(Size::fixed(40))
            .flex_direction(FlexDirection::Column)
            .padding(Padding::all(1))
            .children(children)
            .build(),
    )
    .into_node()
}

fn bench_layout_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("layout_scaling");

    // Benchmark layout computation for N children
    for &size in &[10, 100, 1_000, 5_000, 10_000] {
        let tree = create_view_tree(size);

        group.bench_with_input(
            BenchmarkId::new("compute_layout_flat", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut engine = LayoutEngine::new();
                    let computed = engine.compute_layout(black_box(&tree), 120, 40);
                    black_box(computed);
                });
            },
        );

        group.bench_with_input(BenchmarkId::new("resize_container", size), &size, |b, _| {
            let mut engine = LayoutEngine::new();
            let mut width = 120;
            b.iter(|| {
                width = if width == 120 { 160 } else { 120 };
                let computed = engine.compute_layout(black_box(&tree), black_box(width), 50);
                black_box(computed);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_layout_scaling);
criterion_main!(benches);
