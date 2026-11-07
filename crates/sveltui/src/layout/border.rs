#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Single,
    Rounded,
    Double,
    Thick,
    Ascii,
}

impl BorderStyle {
    pub fn chars(&self) -> Option<BorderChars> {
        match self {
            BorderStyle::None => None,
            BorderStyle::Single => Some(BorderChars {
                top_left: '┌',
                top_right: '┐',
                bottom_left: '└',
                bottom_right: '┘',
                horizontal: '─',
                vertical: '│',
            }),
            BorderStyle::Rounded => Some(BorderChars {
                top_left: '╭',
                top_right: '╮',
                bottom_left: '╰',
                bottom_right: '╯',
                horizontal: '─',
                vertical: '│',
            }),
            BorderStyle::Double => Some(BorderChars {
                top_left: '╔',
                top_right: '╗',
                bottom_left: '╚',
                bottom_right: '╝',
                horizontal: '═',
                vertical: '║',
            }),
            BorderStyle::Thick => Some(BorderChars {
                top_left: '┏',
                top_right: '┓',
                bottom_left: '┗',
                bottom_right: '┛',
                horizontal: '━',
                vertical: '┃',
            }),
            BorderStyle::Ascii => Some(BorderChars {
                top_left: '+',
                top_right: '+',
                bottom_left: '+',
                bottom_right: '+',
                horizontal: '-',
                vertical: '|',
            }),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BorderChars {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
}

/// A side of a box.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl BorderStyle {
    /// The character for a point on a line of this style, given which
    /// directions the line leaves it in. Two directions make a straight
    /// line or a corner, three a tee, four a cross.
    pub fn junction(self, up: bool, down: bool, left: bool, right: bool) -> char {
        // Single and rounded lines only differ at their outer corners.
        let set: [char; 11] = match self {
            BorderStyle::None | BorderStyle::Single | BorderStyle::Rounded => {
                ['─', '│', '┼', '┬', '┴', '├', '┤', '┌', '┐', '└', '┘']
            }
            BorderStyle::Double => ['═', '║', '╬', '╦', '╩', '╠', '╣', '╔', '╗', '╚', '╝'],
            BorderStyle::Thick => ['━', '┃', '╋', '┳', '┻', '┣', '┫', '┏', '┓', '┗', '┛'],
            BorderStyle::Ascii => ['-', '|', '+', '+', '+', '+', '+', '+', '+', '+', '+'],
        };
        let [
            horizontal,
            vertical,
            cross,
            tee_down,
            tee_up,
            tee_right,
            tee_left,
            down_right,
            down_left,
            up_right,
            up_left,
        ] = set;
        match (up, down, left, right) {
            (true, true, true, true) => cross,
            (false, true, true, true) => tee_down,
            (true, false, true, true) => tee_up,
            (true, true, false, true) => tee_right,
            (true, true, true, false) => tee_left,
            (false, true, false, true) => down_right,
            (false, true, true, false) => down_left,
            (true, false, false, true) => up_right,
            (true, false, true, false) => up_left,
            (_, _, false, false) => vertical,
            _ => horizontal,
        }
    }

    /// The character where a rule of style `rule` meets this border on
    /// `side`. Mixed weights use the box-drawing characters made for them
    /// (a single rule meeting a double border is `╤`, not `┬`).
    pub fn tee(self, rule: BorderStyle, side: Side) -> char {
        use BorderStyle::{Ascii, Double, None, Rounded, Single, Thick};
        let light = |style| matches!(style, Single | Rounded | None);
        let [top, bottom, left, right] = match (self, rule) {
            (Ascii, _) | (_, Ascii) => ['+'; 4],
            (Double, r) if light(r) => ['╤', '╧', '╟', '╢'],
            (b, Double) if light(b) => ['╥', '╨', '╞', '╡'],
            (Thick, r) if light(r) => ['┯', '┷', '┠', '┨'],
            (b, Thick) if light(b) => ['┰', '┸', '┝', '┥'],
            (Double, Double) => ['╦', '╩', '╠', '╣'],
            (Thick, Thick) => ['┳', '┻', '┣', '┫'],
            // Double against thick has no joined form; the rule's own tee
            // reads best.
            _ => [
                rule.junction(false, true, true, true),
                rule.junction(true, false, true, true),
                rule.junction(true, true, false, true),
                rule.junction(true, true, true, false),
            ],
        };
        match side {
            Side::Top => top,
            Side::Bottom => bottom,
            Side::Left => left,
            Side::Right => right,
        }
    }
}
