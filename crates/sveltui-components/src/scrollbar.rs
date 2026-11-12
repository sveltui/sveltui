//! A one-cell-wide scrollbar for lists that build only the rows on screen.

/// A column `track` cells tall for `len` rows of which `fit` show, starting
/// at row `first`: `█` for the thumb, `│` for the rest.
pub(crate) fn column(track: usize, fit: usize, len: usize, first: usize) -> String {
    let thumb = (track * fit / len.max(1)).clamp(1, track.max(1));
    let start = (track - thumb) * first / len.saturating_sub(fit).max(1);
    (0..track)
        .map(|i| {
            if (start..start + thumb).contains(&i) {
                "█"
            } else {
                "│"
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
