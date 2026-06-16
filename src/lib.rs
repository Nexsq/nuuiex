mod render;
mod config;

pub use render::canvas::{Box, Canvas, Cell};
pub use render::style::{Border, Color, Modifier, Style};
pub use render::terminal::{Key, Terminal};

pub use config::conf::{self, ConfigError};