pub mod error;
pub mod main;
pub mod settings;
pub mod toosmall;
pub mod editor;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelResult {
    Ok(usize),
    Cancel,
    Quit,
}
