mod config;
pub mod editor;
mod library;
mod panels;
mod render;

pub use render::canvas::{Box, Canvas, Cell};
pub use render::style::{Border, Color, Modifier, Style};
pub use render::terminal::{Key, Terminal};

pub use config::conf;
pub use editor::Editor;
pub use library::lib;
pub use panels::{PanelResult, error, main, settings, toosmall};
