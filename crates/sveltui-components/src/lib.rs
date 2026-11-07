pub mod badge;
pub mod button;
pub mod checkbox;
pub mod combobox;
mod controllable;
pub mod dialog;
pub mod divider;
mod listbox;
pub mod menu;
pub mod popover;
pub mod progress_bar;
pub mod select;
pub mod switch;
pub mod tabs;
pub mod text_area;
pub mod text_buffer;
pub mod text_input;

pub use badge::{Badge, BadgeProps};
pub use button::{Button, ButtonProps};
pub use checkbox::{Checkbox, CheckboxProps};
pub use combobox::{Combobox, ComboboxProps};
/// The combobox parts: `Root`, `Input`, `Trigger`, `Content`, `Item`,
/// `Empty`, `Group`, `GroupHeading` and `Separator`.
#[allow(non_snake_case)]
pub mod Combobox {
    pub use crate::combobox::{
        Content, ContentProps, Empty, EmptyProps, Input, InputProps, Item, ItemProps, Root,
        RootProps, Trigger, TriggerProps,
    };
    pub use crate::listbox::{
        Group, GroupHeading, GroupHeadingProps, GroupProps, Separator, SeparatorProps,
    };
}
pub use dialog::{Dialog, DialogProps};
pub use divider::{Divider, DividerProps};
/// Shared menu primitive and composable parts.
pub use menu as Menu;
#[allow(non_snake_case)]
pub mod DropdownMenu {
    pub use crate::menu::{
        Content, ContentProps, Group, GroupHeading, GroupHeadingProps, GroupProps, Item, ItemProps,
        Root, RootProps, Separator, SeparatorProps, Sub, SubContent, SubContentProps, SubProps,
        SubTrigger, SubTriggerProps, Trigger, TriggerProps,
    };
}
#[allow(non_snake_case)]
pub mod ContextMenu {
    pub use crate::menu::{
        Content, ContentProps, ContextRoot as Root, Group, GroupHeading, GroupHeadingProps,
        GroupProps, Item, ItemProps, RootProps, Separator, SeparatorProps, Sub, SubContent,
        SubContentProps, SubProps, SubTrigger, SubTriggerProps, Trigger, TriggerProps,
    };
}
#[allow(non_snake_case)]
pub mod MenuBar {
    pub use crate::menu::{
        BarMenu as Menu, BarMenuProps as MenuProps, BarRoot as Root, BarRootProps as RootProps,
        Content, ContentProps, Group, GroupHeading, GroupHeadingProps, GroupProps, Item, ItemProps,
        Separator, SeparatorProps, Sub, SubContent, SubContentProps, SubProps, SubTrigger,
        SubTriggerProps, Trigger, TriggerProps,
    };
}
pub use listbox::ListboxOption;
pub use popover::{Popover, PopoverProps};
pub use progress_bar::{ProgressBar, ProgressBarProps};
pub use select::{Select, SelectProps};
/// The select parts: `Root`, `Trigger`, `Value`, `Content`, `Item`, `Group`,
/// `GroupHeading` and `Separator`.
#[allow(non_snake_case)]
pub mod Select {
    pub use crate::listbox::{
        Group, GroupHeading, GroupHeadingProps, GroupProps, Separator, SeparatorProps,
    };
    pub use crate::select::{
        Content, ContentProps, Item, ItemProps, Root, RootProps, Trigger, TriggerProps, Value,
        ValueProps,
    };
}
pub use switch::{Switch, SwitchProps};
/// Composable tabs: `Tabs::Root`, `Tabs::List`, `Tabs::Trigger`, `Tabs::Content`.
#[allow(non_snake_case)]
pub mod Tabs {
    pub use crate::tabs::{
        Content, ContentProps, List, ListProps, Root, RootProps, Trigger, TriggerProps,
    };
}
pub use tabs::TabsActivation;
pub use text_area::{TextArea, TextAreaProps};
pub use text_buffer::*;
pub use text_input::{TextInput, TextInputProps, get_cursor, set_cursor};
