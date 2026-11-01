use sveltui::prelude::*;

#[derive(Clone)]
pub enum CodeBinding {
    State(State<String>),
    Writable(WriteSignal<String>),
    Read(ReadSignal<String>),
    Pair(ReadSignal<String>, WriteSignal<String>),
    Static(String),
}

impl CodeBinding {
    pub fn read_signal(&self) -> Option<ReadSignal<String>> {
        match self {
            Self::State(s) => Some(s.read()),
            Self::Read(r) | Self::Pair(r, _) => Some(*r),
            _ => None,
        }
    }

    pub fn write_signal(&self) -> Option<WriteSignal<String>> {
        match self {
            Self::State(s) => Some(s.write()),
            Self::Writable(w) | Self::Pair(_, w) => Some(*w),
            _ => None,
        }
    }

    pub fn get_text(&self) -> String {
        match self {
            Self::State(s) => s.get(),
            Self::Read(r) | Self::Pair(r, _) => r.get(),
            Self::Writable(_) => String::new(),
            Self::Static(s) => s.clone(),
        }
    }
}

impl From<State<String>> for CodeBinding {
    fn from(s: State<String>) -> Self {
        Self::State(s)
    }
}

impl From<WriteSignal<String>> for CodeBinding {
    fn from(w: WriteSignal<String>) -> Self {
        Self::Writable(w)
    }
}

impl From<ReadSignal<String>> for CodeBinding {
    fn from(r: ReadSignal<String>) -> Self {
        Self::Read(r)
    }
}

impl From<(ReadSignal<String>, WriteSignal<String>)> for CodeBinding {
    fn from(pair: (ReadSignal<String>, WriteSignal<String>)) -> Self {
        Self::Pair(pair.0, pair.1)
    }
}

impl From<String> for CodeBinding {
    fn from(s: String) -> Self {
        Self::Static(s)
    }
}

impl From<&str> for CodeBinding {
    fn from(s: &str) -> Self {
        Self::Static(s.to_string())
    }
}

impl IntoPropOption<CodeBinding> for CodeBinding {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(self)
    }
}

impl IntoPropOption<CodeBinding> for State<String> {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(CodeBinding::State(self))
    }
}

impl IntoPropOption<CodeBinding> for WriteSignal<String> {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(CodeBinding::Writable(self))
    }
}

impl IntoPropOption<CodeBinding> for ReadSignal<String> {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(CodeBinding::Read(self))
    }
}

impl IntoPropOption<CodeBinding> for (ReadSignal<String>, WriteSignal<String>) {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(CodeBinding::Pair(self.0, self.1))
    }
}

impl IntoPropOption<CodeBinding> for String {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(CodeBinding::Static(self))
    }
}

impl IntoPropOption<CodeBinding> for &'static str {
    fn into_prop_option(self) -> Option<CodeBinding> {
        Some(CodeBinding::Static(self.to_string()))
    }
}
