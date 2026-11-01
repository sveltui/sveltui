#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Margin {
    pub top: u16,
    pub right: u16,
    pub bottom: u16,
    pub left: u16,
}

impl Margin {
    pub const fn zero() -> Self {
        Self {
            top: 0,
            right: 0,
            bottom: 0,
            left: 0,
        }
    }

    pub const fn all(val: u16) -> Self {
        Self {
            top: val,
            right: val,
            bottom: val,
            left: val,
        }
    }

    pub const fn symmetric(horizontal: u16, vertical: u16) -> Self {
        Self {
            top: vertical,
            right: horizontal,
            bottom: vertical,
            left: horizontal,
        }
    }

    pub const fn only(top: u16, right: u16, bottom: u16, left: u16) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl From<u16> for Margin {
    fn from(val: u16) -> Self {
        Margin::all(val)
    }
}

impl From<(u16, u16)> for Margin {
    fn from((horizontal, vertical): (u16, u16)) -> Self {
        Margin::symmetric(horizontal, vertical)
    }
}

impl From<(u16, u16, u16, u16)> for Margin {
    fn from((top, right, bottom, left): (u16, u16, u16, u16)) -> Self {
        Margin::only(top, right, bottom, left)
    }
}
