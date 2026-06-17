mod config;
mod render;
mod tabs;

pub use render::canvas::{Box, Canvas, Cell};
pub use render::style::{Border, Color, Modifier, Style};
pub use render::terminal::{Key, Terminal};

pub use config::conf::{self, ConfigError};

pub use tabs::toosmall;
