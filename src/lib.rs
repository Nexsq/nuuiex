mod config;
pub mod engine;
mod library;
mod panels;
mod render;
mod theme;

pub use render::canvas::{Box, Canvas, Cell};
pub use render::style::{Border, Color, Gradient, Modifier, Style};
pub use render::terminal::{HAS_FOCUS, Key, Terminal};

pub use config::conf;
pub use engine::core::EngineMessage;
pub use library::lib;
pub use panels::{editor, error, main, result::PanelResult, settings, toosmall};
pub use theme::themecore;

pub fn get_config_dir() -> Result<std::path::PathBuf, String> {
    let path = {
        #[cfg(feature = "portable")]
        {
            if let Ok(mut exe_path) = std::env::current_exe() {
                exe_path.pop();
                exe_path.join(".nuui")
            } else {
                std::env::current_dir().unwrap_or_default().join(".nuui")
            }
        }
        #[cfg(not(feature = "portable"))]
        {
            directories::ProjectDirs::from("com", "Nexsq", "nuui")
                .map(|d| d.config_dir().to_path_buf())
                .ok_or_else(|| "Failed to locate the system configuration directory.".to_string())?
        }
    };

    if !path.exists() {
        if std::fs::create_dir_all(&path).is_ok() {
            #[cfg(all(windows, feature = "portable"))]
            {
                use std::os::windows::process::CommandExt;
                let _ = std::process::Command::new("attrib")
                    .arg("+h")
                    .arg(&path)
                    .creation_flags(0x08000000)
                    .status();
            }
        }
    }

    Ok(path)
}
