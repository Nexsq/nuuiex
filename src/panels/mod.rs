pub mod editor;
pub mod error;
pub mod main;
pub mod settings;
pub mod toosmall;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelResult {
    Ok(usize),
    Cancel,
    Quit,
}
