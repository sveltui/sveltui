use std::io::{self, Write};

/// The mouse pointer's shape over an element, named after the CSS `cursor`
/// values. This is the pointer, not the text caret (see `CursorStyle`).
///
/// Terminals show it through the OSC 22 escape; ones that don't know it
/// (most of the older ones) ignore it and keep their usual pointer.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PointerShape {
    #[default]
    Default,
    Pointer,
    Text,
    Crosshair,
    Move,
    Grab,
    Grabbing,
    /// Resizing across columns, such as a vertical split handle.
    ColResize,
    /// Resizing across rows, such as a horizontal split handle.
    RowResize,
    EwResize,
    NsResize,
    NotAllowed,
    Wait,
}

/// Which names a terminal looks pointer shapes up by. There is no standard:
/// kitty has its own list without `col-resize` or `row-resize`, xterm takes
/// X11 cursor names, and most others (Ghostty, foot, WezTerm) take CSS names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PointerDialect {
    #[default]
    Css,
    Kitty,
    Xterm,
}

impl PointerDialect {
    /// Picks the dialect from the environment. `SVELTUI_POINTER_DIALECT`
    /// (`css`, `kitty` or `xterm`) overrides detection.
    pub fn detect() -> Self {
        Self::from_env(|key| std::env::var(key).ok())
    }

    pub(crate) fn from_env(get: impl Fn(&str) -> Option<String>) -> Self {
        if let Some(forced) = get("SVELTUI_POINTER_DIALECT") {
            match forced.to_ascii_lowercase().as_str() {
                "kitty" => return Self::Kitty,
                "xterm" => return Self::Xterm,
                "css" => return Self::Css,
                _ => {}
            }
        }
        let term = get("TERM").unwrap_or_default();
        if term == "xterm-kitty" || get("KITTY_WINDOW_ID").is_some() {
            Self::Kitty
        } else if get("XTERM_VERSION").is_some() && !term.contains("ghostty") {
            Self::Xterm
        } else {
            Self::Css
        }
    }
}

impl PointerShape {
    /// The CSS name, which is also what Ghostty, foot and WezTerm take.
    pub const fn css_name(self) -> &'static str {
        match self {
            PointerShape::Default => "default",
            PointerShape::Pointer => "pointer",
            PointerShape::Text => "text",
            PointerShape::Crosshair => "crosshair",
            PointerShape::Move => "move",
            PointerShape::Grab => "grab",
            PointerShape::Grabbing => "grabbing",
            PointerShape::ColResize => "col-resize",
            PointerShape::RowResize => "row-resize",
            PointerShape::EwResize => "ew-resize",
            PointerShape::NsResize => "ns-resize",
            PointerShape::NotAllowed => "not-allowed",
            PointerShape::Wait => "wait",
        }
    }

    /// The name `dialect` knows this shape by.
    pub const fn name(self, dialect: PointerDialect) -> &'static str {
        match dialect {
            PointerDialect::Css => self.css_name(),
            // Kitty's list has no `col-resize`/`row-resize`; its
            // `ew-resize`/`ns-resize` are the bar-and-arrows cursors.
            PointerDialect::Kitty => match self {
                PointerShape::ColResize => "ew-resize",
                PointerShape::RowResize => "ns-resize",
                other => other.css_name(),
            },
            PointerDialect::Xterm => match self {
                PointerShape::Default => "left_ptr",
                PointerShape::Pointer => "hand2",
                PointerShape::Text => "xterm",
                PointerShape::Crosshair => "crosshair",
                PointerShape::Move => "fleur",
                PointerShape::Grab | PointerShape::Grabbing => "hand1",
                PointerShape::ColResize | PointerShape::EwResize => "sb_h_double_arrow",
                PointerShape::RowResize | PointerShape::NsResize => "sb_v_double_arrow",
                PointerShape::NotAllowed => "X_cursor",
                PointerShape::Wait => "watch",
            },
        }
    }
}

/// Emits OSC 22 for `shape`, terminated with ST.
pub(crate) fn write_pointer_shape(
    mut out: impl Write,
    shape: PointerShape,
    dialect: PointerDialect,
) -> io::Result<()> {
    write!(out, "\x1b]22;{}\x1b\\", shape.name(dialect))?;
    out.flush()
}
