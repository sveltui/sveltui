use crossterm::cursor::SetCursorStyle;

/// Native terminal cursor shapes and styles supported by crossterm.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CursorStyle {
    /// Default terminal cursor shape configured by the user/terminal emulator.
    #[default]
    Default,
    /// Block cursor (blinking).
    Block,
    /// Steady (non-blinking) block cursor.
    SteadyBlock,
    /// Blinking block cursor.
    BlinkingBlock,
    /// Vertical line / bar cursor (blinking).
    Line,
    /// Steady vertical line / bar cursor.
    SteadyLine,
    /// Blinking vertical line / bar cursor.
    BlinkingLine,
    /// Bar cursor alias for Line.
    Bar,
    /// Steady bar cursor alias for SteadyLine.
    SteadyBar,
    /// Blinking bar cursor alias for BlinkingLine.
    BlinkingBar,
    /// Underline cursor (blinking).
    Underline,
    /// Steady underline cursor.
    SteadyUnderline,
    /// Blinking underline cursor.
    BlinkingUnderline,
}

impl CursorStyle {
    /// Convert to crossterm's native `SetCursorStyle` command.
    pub fn to_crossterm(self) -> SetCursorStyle {
        match self {
            CursorStyle::Default => SetCursorStyle::DefaultUserShape,
            CursorStyle::Block | CursorStyle::BlinkingBlock => SetCursorStyle::BlinkingBlock,
            CursorStyle::SteadyBlock => SetCursorStyle::SteadyBlock,
            CursorStyle::Line
            | CursorStyle::BlinkingLine
            | CursorStyle::Bar
            | CursorStyle::BlinkingBar => SetCursorStyle::BlinkingBar,
            CursorStyle::SteadyLine | CursorStyle::SteadyBar => SetCursorStyle::SteadyBar,
            CursorStyle::Underline | CursorStyle::BlinkingUnderline => {
                SetCursorStyle::BlinkingUnderScore
            }
            CursorStyle::SteadyUnderline => SetCursorStyle::SteadyUnderScore,
        }
    }
}
