use super::*;

fn sizing(track: Track) -> TrackSizingFunction {
    track.sizing()
}

#[test]
fn fixed_tracks_have_the_same_bounds() {
    let size = sizing(length(20));
    assert_eq!(size.min, MinTrackSizingFunction::length(20.0));
    assert_eq!(size.max, MaxTrackSizingFunction::length(20.0));
}

#[test]
fn a_fraction_has_an_auto_lower_bound_like_css() {
    let size = sizing(fr(2.0));
    assert_eq!(size.min, MinTrackSizingFunction::auto());
    assert_eq!(size.max, MaxTrackSizingFunction::fr(2.0));
}

#[test]
fn minmax_takes_each_bound_from_its_side() {
    let size = sizing(minmax(length(10), fr(1.0)));
    assert_eq!(size.min, MinTrackSizingFunction::length(10.0));
    assert_eq!(size.max, MaxTrackSizingFunction::fr(1.0));
}

#[test]
fn templates_keep_repeats_as_repeats() {
    let template: Vec<GridTemplateComponent<String>> = to_template(&[
        length(20),
        repeat(3, [fr(1.0)]),
        repeat(Repeat::AutoFill, [length(8)]),
    ]);
    assert_eq!(template.len(), 3);
    assert!(matches!(&template[1], GridTemplateComponent::Repeat(r)
        if r.count == RepetitionCount::Count(3) && r.tracks.len() == 1));
    assert!(matches!(&template[2], GridTemplateComponent::Repeat(r)
        if r.count == RepetitionCount::AutoFill));
}

#[test]
fn auto_tracks_spell_out_counted_repeats_and_drop_the_rest() {
    let tracks = to_auto_tracks(&[
        length(1),
        repeat(2, [length(2), length(3)]),
        repeat(Repeat::AutoFit, [length(9)]),
    ]);
    let lengths: Vec<_> = tracks.iter().map(|t| t.max).collect();
    assert_eq!(
        lengths,
        [1.0, 2.0, 3.0, 2.0, 3.0].map(MaxTrackSizingFunction::length)
    );
}

#[test]
fn placement_helpers_set_the_right_edges() {
    assert_eq!(
        span(2),
        GridLines {
            start: GridLine::Auto,
            end: GridLine::Span(2)
        }
    );
    assert_eq!(
        line(3),
        GridLines {
            start: GridLine::Line(3),
            end: GridLine::Auto
        }
    );
    assert_eq!(GridLines::from((1, -1)), lines(1, -1));
}
