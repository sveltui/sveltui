/// How an image reaches the terminal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ImageProtocol {
    /// The best protocol this build has and the terminal speaks, else
    /// half-block cells.
    #[default]
    Auto,
    /// The kitty graphics protocol (kitty, Ghostty, WezTerm). Needs the
    /// `kitty` feature.
    Kitty,
    /// iTerm2 inline images (iTerm2, WezTerm). Needs the `iterm2` feature.
    Iterm2,
    /// Sixel (foot, xterm, mlterm and others). Needs the `sixel` feature.
    Sixel,
    /// Half-block cells, which work in any terminal with truecolor.
    HalfBlock,
}

impl ImageProtocol {
    /// Whether this build can speak the protocol. Half blocks always can.
    pub const fn is_compiled_in(self) -> bool {
        match self {
            ImageProtocol::Auto | ImageProtocol::HalfBlock => true,
            ImageProtocol::Kitty => cfg!(feature = "kitty"),
            ImageProtocol::Iterm2 => cfg!(feature = "iterm2"),
            ImageProtocol::Sixel => cfg!(feature = "sixel"),
        }
    }

    /// What `self` comes to in this terminal: never `Auto`, and
    /// `HalfBlock` when the protocol isn't compiled in.
    pub fn resolve(self) -> Self {
        self.resolve_with(|key| std::env::var(key).ok())
    }

    pub(crate) fn resolve_with(self, get: impl Fn(&str) -> Option<String>) -> Self {
        match self {
            ImageProtocol::Auto => Self::candidates(&get)
                .into_iter()
                .find(|p| p.is_compiled_in())
                .unwrap_or(ImageProtocol::HalfBlock),
            forced if forced.is_compiled_in() => forced,
            _ => ImageProtocol::HalfBlock,
        }
    }

    /// The protocols this terminal speaks, best first. The environment
    /// variable `SVELTUI_IMAGE_PROTOCOL` (`kitty`, `iterm2`, `sixel` or
    /// `halfblock`) overrides detection.
    fn candidates(get: &impl Fn(&str) -> Option<String>) -> Vec<Self> {
        if let Some(forced) = get("SVELTUI_IMAGE_PROTOCOL") {
            match forced.to_ascii_lowercase().as_str() {
                "kitty" => return vec![ImageProtocol::Kitty],
                "iterm2" => return vec![ImageProtocol::Iterm2],
                "sixel" => return vec![ImageProtocol::Sixel],
                "halfblock" | "half-block" => return vec![ImageProtocol::HalfBlock],
                _ => {}
            }
        }
        let term = get("TERM").unwrap_or_default();
        let program = get("TERM_PROGRAM").unwrap_or_default();
        if program == "WezTerm" {
            vec![
                ImageProtocol::Kitty,
                ImageProtocol::Iterm2,
                ImageProtocol::Sixel,
            ]
        } else if get("KITTY_WINDOW_ID").is_some()
            || term == "xterm-kitty"
            || term == "xterm-ghostty"
            || program == "ghostty"
        {
            vec![ImageProtocol::Kitty]
        } else if program == "iTerm.app" {
            vec![ImageProtocol::Iterm2]
        } else if term.starts_with("foot")
            || term.starts_with("mlterm")
            || term.starts_with("yaft")
            || get("XTERM_VERSION").is_some()
        {
            vec![ImageProtocol::Sixel]
        } else {
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(pairs: &'static [(&'static str, &'static str)]) -> impl Fn(&str) -> Option<String> {
        move |key| {
            pairs
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| (*v).to_owned())
        }
    }

    #[test]
    fn terminals_are_recognised_from_their_environment() {
        let kitty = env(&[("TERM", "xterm-kitty")]);
        assert_eq!(ImageProtocol::candidates(&kitty), [ImageProtocol::Kitty]);
        let ghostty = env(&[("TERM_PROGRAM", "ghostty")]);
        assert_eq!(ImageProtocol::candidates(&ghostty), [ImageProtocol::Kitty]);
        let iterm = env(&[("TERM_PROGRAM", "iTerm.app")]);
        assert_eq!(ImageProtocol::candidates(&iterm), [ImageProtocol::Iterm2]);
        let foot = env(&[("TERM", "foot")]);
        assert_eq!(ImageProtocol::candidates(&foot), [ImageProtocol::Sixel]);
        assert!(ImageProtocol::candidates(&env(&[("TERM", "xterm-256color")])).is_empty());
    }

    #[test]
    fn an_unknown_terminal_gets_half_blocks() {
        let plain = env(&[("TERM", "xterm-256color")]);
        assert_eq!(
            ImageProtocol::Auto.resolve_with(plain),
            ImageProtocol::HalfBlock
        );
    }

    #[test]
    fn the_environment_override_wins_when_the_protocol_is_compiled_in() {
        let forced = env(&[
            ("TERM", "xterm-kitty"),
            ("SVELTUI_IMAGE_PROTOCOL", "halfblock"),
        ]);
        assert_eq!(
            ImageProtocol::Auto.resolve_with(forced),
            ImageProtocol::HalfBlock
        );
    }

    #[cfg(feature = "kitty")]
    #[test]
    fn kitty_is_picked_in_kitty_when_compiled_in() {
        let kitty = env(&[("KITTY_WINDOW_ID", "1")]);
        assert_eq!(
            ImageProtocol::Auto.resolve_with(kitty),
            ImageProtocol::Kitty
        );
    }

    #[cfg(not(feature = "sixel"))]
    #[test]
    fn a_protocol_that_is_not_compiled_in_falls_back_to_half_blocks() {
        let foot = env(&[("TERM", "foot")]);
        assert_eq!(
            ImageProtocol::Auto.resolve_with(foot),
            ImageProtocol::HalfBlock
        );
        assert_eq!(
            ImageProtocol::Sixel.resolve_with(env(&[])),
            ImageProtocol::HalfBlock
        );
    }
}
