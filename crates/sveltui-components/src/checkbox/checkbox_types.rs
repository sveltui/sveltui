use sveltui::prelude::*;

#[derive(Clone, Copy)]
pub enum CheckboxBinding {
    Read(ReadSignal<bool>),
    State(State<bool>),
}

impl From<ReadSignal<bool>> for CheckboxBinding {
    fn from(s: ReadSignal<bool>) -> Self {
        Self::Read(s)
    }
}

impl From<State<bool>> for CheckboxBinding {
    fn from(s: State<bool>) -> Self {
        Self::State(s)
    }
}

impl CheckboxBinding {
    pub fn get(&self) -> bool {
        match self {
            Self::Read(r) => r.get(),
            Self::State(s) => s.get(),
        }
    }
}

impl IntoPropOption<CheckboxBinding> for CheckboxBinding {
    fn into_prop_option(self) -> Option<CheckboxBinding> {
        Some(self)
    }
}

impl IntoPropOption<CheckboxBinding> for ReadSignal<bool> {
    fn into_prop_option(self) -> Option<CheckboxBinding> {
        Some(CheckboxBinding::Read(self))
    }
}

impl IntoPropOption<CheckboxBinding> for State<bool> {
    fn into_prop_option(self) -> Option<CheckboxBinding> {
        Some(CheckboxBinding::State(self))
    }
}

#[derive(Props)]
pub struct CheckboxProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub checked: Option<CheckboxBinding>,
    #[prop(optional)]
    pub set_checked: Option<WriteSignal<bool>>,
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
