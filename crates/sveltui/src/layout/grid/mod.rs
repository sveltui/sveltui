//! Grid layout: the sizes of a grid's columns and rows (`Track`), and where
//! a child sits in it (`GridLines`). These are our own types, converted to
//! taffy's at the edge, so code using them doesn't change when taffy does.

use taffy::style::{
    GridPlacement as TaffyPlacement, GridTemplateComponent, GridTemplateRepetition,
    MaxTrackSizingFunction, MinTrackSizingFunction, RepetitionCount, TrackSizingFunction,
};

#[cfg(test)]
mod grid_tests;

pub use taffy::style::GridAutoFlow;

/// The size of one column or row in a grid.
///
/// We usually build these with the helpers: `length(20)`, `fr(1.0)`,
/// `auto()`, `minmax(length(10), fr(1.0))`, `repeat(3, [fr(1.0)])`.
#[derive(Clone, Debug, PartialEq)]
pub enum Track {
    /// A fixed number of cells.
    Length(u16),
    /// A fraction of the grid's size, where `1.0` is all of it.
    Percent(f32),
    /// A share of the space left once the other tracks are sized.
    Fr(f32),
    /// Sized from content, and stretched to fill leftover space.
    Auto,
    /// As narrow as the content allows (its longest word, for text).
    MinContent,
    /// As wide as the content wants, without wrapping.
    MaxContent,
    /// Sized from content, but never past this many cells.
    FitContent(u16),
    /// Never smaller than the first track, never bigger than the second.
    MinMax(Box<Track>, Box<Track>),
    /// The tracks repeated. Only grid templates can repeat; in
    /// `grid_auto_rows`/`grid_auto_columns` a counted repeat is spelled
    /// out and an auto-fill/auto-fit one is ignored.
    Repeat(Repeat, Vec<Track>),
}

/// How many times `repeat` lays out its tracks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Repeat {
    Count(u16),
    /// As many times as fit, keeping empty repetitions.
    AutoFill,
    /// As many times as fit, collapsing the empty ones.
    AutoFit,
}

impl From<u16> for Repeat {
    fn from(count: u16) -> Self {
        Repeat::Count(count)
    }
}

pub fn length(cells: u16) -> Track {
    Track::Length(cells)
}

pub fn percent(fraction: f32) -> Track {
    Track::Percent(fraction)
}

pub fn fr(share: f32) -> Track {
    Track::Fr(share)
}

pub fn auto() -> Track {
    Track::Auto
}

pub fn min_content() -> Track {
    Track::MinContent
}

pub fn max_content() -> Track {
    Track::MaxContent
}

pub fn fit_content(cells: u16) -> Track {
    Track::FitContent(cells)
}

pub fn minmax(min: Track, max: Track) -> Track {
    Track::MinMax(Box::new(min), Box::new(max))
}

/// `tracks`, `count` times over (or `Repeat::AutoFill`/`AutoFit` times).
pub fn repeat(count: impl Into<Repeat>, tracks: impl IntoIterator<Item = Track>) -> Track {
    Track::Repeat(count.into(), tracks.into_iter().collect())
}

/// `count` equal columns or rows.
pub fn evenly(count: u16) -> Track {
    repeat(count, [fr(1.0)])
}

impl Track {
    /// The lower bound of this track when it's used inside `minmax`, or on
    /// its own. A fraction can't be a lower bound, so it becomes `auto`, as
    /// in CSS.
    fn min(&self) -> MinTrackSizingFunction {
        match self {
            Track::Length(cells) => MinTrackSizingFunction::length(f32::from(*cells)),
            Track::Percent(fraction) => MinTrackSizingFunction::percent(*fraction),
            Track::MinContent => MinTrackSizingFunction::min_content(),
            Track::MaxContent => MinTrackSizingFunction::max_content(),
            Track::MinMax(min, _) => min.min(),
            Track::Fr(_) | Track::Auto | Track::FitContent(_) | Track::Repeat(..) => {
                MinTrackSizingFunction::auto()
            }
        }
    }

    fn max(&self) -> MaxTrackSizingFunction {
        match self {
            Track::Length(cells) => MaxTrackSizingFunction::length(f32::from(*cells)),
            Track::Percent(fraction) => MaxTrackSizingFunction::percent(*fraction),
            Track::Fr(share) => MaxTrackSizingFunction::fr(*share),
            Track::MinContent => MaxTrackSizingFunction::min_content(),
            Track::MaxContent => MaxTrackSizingFunction::max_content(),
            Track::FitContent(cells) => MaxTrackSizingFunction::fit_content_px(f32::from(*cells)),
            Track::MinMax(_, max) => max.max(),
            Track::Auto | Track::Repeat(..) => MaxTrackSizingFunction::auto(),
        }
    }

    fn sizing(&self) -> TrackSizingFunction {
        TrackSizingFunction {
            min: self.min(),
            max: self.max(),
        }
    }
}

/// A grid template: the explicit columns or rows.
pub(crate) fn to_template<S: taffy::style::CheapCloneStr>(
    tracks: &[Track],
) -> Vec<GridTemplateComponent<S>> {
    tracks
        .iter()
        .map(|track| match track {
            Track::Repeat(count, inner) => GridTemplateComponent::Repeat(GridTemplateRepetition {
                count: match count {
                    Repeat::Count(n) => RepetitionCount::Count(*n),
                    Repeat::AutoFill => RepetitionCount::AutoFill,
                    Repeat::AutoFit => RepetitionCount::AutoFit,
                },
                tracks: inner.iter().map(Track::sizing).collect(),
                line_names: Vec::new(),
            }),
            other => GridTemplateComponent::Single(other.sizing()),
        })
        .collect()
}

/// Sizes for rows or columns the grid adds on its own.
pub(crate) fn to_auto_tracks(tracks: &[Track]) -> Vec<TrackSizingFunction> {
    let mut sizes = Vec::new();
    for track in tracks {
        match track {
            Track::Repeat(Repeat::Count(n), inner) => {
                for _ in 0..*n {
                    sizes.extend(inner.iter().map(Track::sizing));
                }
            }
            Track::Repeat(..) => {}
            other => sizes.push(other.sizing()),
        }
    }
    sizes
}

/// Anything that can be a list of tracks: an array, a `Vec` or a slice of
/// `Track`s, or a single `Track`.
pub trait IntoTracks {
    fn into_tracks(self) -> Vec<Track>;
}

impl IntoTracks for Track {
    fn into_tracks(self) -> Vec<Track> {
        vec![self]
    }
}

impl IntoTracks for Vec<Track> {
    fn into_tracks(self) -> Vec<Track> {
        self
    }
}

impl<const N: usize> IntoTracks for [Track; N] {
    fn into_tracks(self) -> Vec<Track> {
        self.into()
    }
}

impl IntoTracks for &[Track] {
    fn into_tracks(self) -> Vec<Track> {
        self.to_vec()
    }
}

/// One edge of a child's area in the grid.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum GridLine {
    /// Wherever auto-placement puts it.
    #[default]
    Auto,
    /// A numbered grid line, starting from 1. Negative numbers count back
    /// from the end, so `-1` is the last line.
    Line(i16),
    /// So many tracks on from the other edge.
    Span(u16),
}

impl GridLine {
    fn to_taffy(self) -> TaffyPlacement {
        match self {
            GridLine::Auto => TaffyPlacement::Auto,
            GridLine::Line(n) => TaffyPlacement::Line(n.into()),
            GridLine::Span(n) => TaffyPlacement::Span(n),
        }
    }
}

/// Where a child sits along one axis of its grid: from `start` to `end`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GridLines {
    pub start: GridLine,
    pub end: GridLine,
}

impl GridLines {
    pub(crate) fn to_taffy(self) -> taffy::geometry::Line<TaffyPlacement> {
        taffy::geometry::Line {
            start: self.start.to_taffy(),
            end: self.end.to_taffy(),
        }
    }
}

/// Spans `tracks` tracks, wherever auto-placement starts it.
pub fn span(tracks: u16) -> GridLines {
    GridLines {
        start: GridLine::Auto,
        end: GridLine::Span(tracks),
    }
}

/// Starts at grid line `n` and takes one track.
pub fn line(n: i16) -> GridLines {
    GridLines {
        start: GridLine::Line(n),
        end: GridLine::Auto,
    }
}

/// From grid line `start` to grid line `end`.
pub fn lines(start: i16, end: i16) -> GridLines {
    GridLines {
        start: GridLine::Line(start),
        end: GridLine::Line(end),
    }
}

impl From<i16> for GridLines {
    fn from(n: i16) -> Self {
        line(n)
    }
}

impl From<(i16, i16)> for GridLines {
    fn from((start, end): (i16, i16)) -> Self {
        lines(start, end)
    }
}
