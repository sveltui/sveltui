//! Times a whole frame the way `App` draws one: layout, render and hit
//! targets, through `render_node_interactive`. The layout benchmarks only
//! time `compute_layout`, so work that render repeats doesn't show up there.

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use sveltui::element::taffy_bridge::layout;
use sveltui::element::{HitMap, render_node_interactive};
use sveltui::layout::{FlexDirection, Rect};
use sveltui::prelude::*;
use sveltui::{Buffer, view};

const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;

/// A settings-style list: each row is a label and a value side by side.
fn rows(count: usize) -> VNode {
    let items: Vec<VNode> = (0..count)
        .map(|i| {
            view! {
                <View flex_direction=FlexDirection::Row gap=1>
                    <TextSpan>{format!("Setting {i}")}</TextSpan>
                    <TextSpan foreground_color=Color::DarkGrey>{format!("value {i}")}</TextSpan>
                </View>
            }
            .into_node()
        })
        .collect();
    view! { <View width=WIDTH>{items}</View> }.into_node()
}

/// Wrapped paragraphs, where text measuring dominates.
fn paragraphs(count: usize) -> VNode {
    let text = "Once upon a midnight dreary, while I pondered, weak and weary, \
                over many a quaint and curious volume of forgotten lore.";
    let items: Vec<VNode> = (0..count)
        .map(|_| view! { <Paragraph>{text}</Paragraph> }.into_node())
        .collect();
    view! { <View width=60>{items}</View> }.into_node()
}

/// Bordered cards holding pressable rows, several levels deep.
fn cards(count: usize) -> VNode {
    let items: Vec<VNode> = (0..count)
        .map(|i| {
            view! {
                <View border=BorderStyle::Single padding=1>
                    <View flex_direction=FlexDirection::Row>
                        <Pressable on_click={|| {}}>
                            <TextSpan>{format!("Card {i}")}</TextSpan>
                            <TextSpan>"details"</TextSpan>
                        </Pressable>
                    </View>
                </View>
            }
            .into_node()
        })
        .collect();
    view! { <View width=WIDTH>{items}</View> }.into_node()
}

/// A named builder for one kind of tree, sized by its item count.
type Shape = (&'static str, fn(usize) -> VNode);

fn bench_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame");
    let shapes: [Shape; 3] = [("rows", rows), ("paragraphs", paragraphs), ("cards", cards)];
    for (name, build) in shapes {
        for count in [30, 300, 3_000] {
            let tree = build(count);
            group.bench_with_input(BenchmarkId::new(name, count), &tree, |b, tree| {
                b.iter(|| {
                    let mut buffer = Buffer::new(WIDTH, HEIGHT);
                    let mut hits = HitMap::new();
                    render_node_interactive(
                        black_box(tree),
                        Rect::new(0, 0, WIDTH, HEIGHT),
                        &mut buffer,
                        &mut hits,
                    );
                    black_box(buffer);
                });
            });
        }
    }
    group.finish();
}

/// The same trees split into the frame's main steps, to show where the
/// time goes.
fn bench_frame_parts(c: &mut Criterion) {
    let mut group = c.benchmark_group("frame_parts");
    for (name, tree) in [("rows", rows(300)), ("cards", cards(300))] {
        group.bench_function(BenchmarkId::new("layout", name), |b| {
            b.iter(|| black_box(layout(black_box(&tree), WIDTH, HEIGHT)));
        });
        group.bench_function(BenchmarkId::new("interactions", name), |b| {
            b.iter(|| {
                let mut hits = HitMap::new();
                hits.collect_interactions(black_box(&tree));
                black_box(hits);
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench_frame, bench_frame_parts);
criterion_main!(benches);
