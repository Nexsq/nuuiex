#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PanelResult {
    Ok(usize),
    Cancel,
    Quit,
}
