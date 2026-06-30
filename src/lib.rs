mod config;
mod panels;
mod render;

pub use render::canvas::{Box, Canvas, Cell};
pub use render::style::{Border, Color, Modifier, Style};
pub use render::terminal::{Key, Terminal};

pub use config::conf;
pub use panels::{main, toosmall};
