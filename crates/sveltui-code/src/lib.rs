pub mod binding;
pub mod buffer;
pub mod code_editor;
pub mod code_preview;
pub mod highlight;

pub use binding::CodeBinding;
pub use buffer::{
    CodeBuffer, get_code_buffer, set_code_buffer, with_code_buffer, with_code_buffer_mut,
};
pub use code_editor::{CodeEditor, CodeEditorProps};
pub use code_preview::{CodePreview, CodePreviewProps};
pub use highlight::{
    DEFAULT_THEME, HighlightedSpan, clip_spans, find_syntax, find_theme, highlight_text,
    syntax_set, syntect_color_to_crossterm, theme_set,
};
pub use sveltui::terminal::CursorStyle;

pub mod prelude {
    pub use crate::binding::CodeBinding;
    pub use crate::buffer::{
        CodeBuffer, get_code_buffer, set_code_buffer, with_code_buffer, with_code_buffer_mut,
    };
    pub use crate::code_editor::{CodeEditor, CodeEditorProps};
    pub use crate::code_preview::{CodePreview, CodePreviewProps};
    pub use crate::highlight::HighlightedSpan;
    pub use sveltui::terminal::CursorStyle;
}
