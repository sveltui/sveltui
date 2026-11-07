use sveltui::prelude::*;

#[derive(Clone, Copy)]
pub enum SwitchBinding {
    Read(ReadSignal<bool>),
    State(State<bool>),
}

impl From<ReadSignal<bool>> for SwitchBinding {
    fn from(s: ReadSignal<bool>) -> Self {
        Self::Read(s)
    }
}

impl From<State<bool>> for SwitchBinding {
    fn from(s: State<bool>) -> Self {
        Self::State(s)
    }
}

impl SwitchBinding {
    pub fn get(&self) -> bool {
        match self {
            Self::Read(r) => r.get(),
            Self::State(s) => s.get(),
        }
    }
}

impl IntoPropOption<SwitchBinding> for SwitchBinding {
    fn into_prop_option(self) -> Option<SwitchBinding> {
        Some(self)
    }
}

impl IntoPropOption<SwitchBinding> for ReadSignal<bool> {
    fn into_prop_option(self) -> Option<SwitchBinding> {
        Some(SwitchBinding::Read(self))
    }
}

impl IntoPropOption<SwitchBinding> for State<bool> {
    fn into_prop_option(self) -> Option<SwitchBinding> {
        Some(SwitchBinding::State(self))
    }
}

#[derive(Props)]
pub struct SwitchProps {
    #[prop(optional)]
    pub id: Option<FocusId>,
    #[prop(optional)]
    pub checked: Option<SwitchBinding>,
    #[prop(optional)]
    pub set_checked: Option<WriteSignal<bool>>,
    #[prop(optional)]
    pub label: Option<String>,
    #[prop(default = false)]
    pub disabled: bool,
    #[snippet(optional)]
    pub children: Option<Snippet>,
}
