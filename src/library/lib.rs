use directories::ProjectDirs;
use std::cmp::Ordering;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum MacroNode {
    Script {
        name: String,
        path: PathBuf,
    },
    Folder {
        name: String,
        path: PathBuf,
        children: Vec<MacroNode>,
    },
}

impl MacroNode {
    pub fn name(&self) -> &str {
        match self {
            Self::Script { name, .. } => name,
            Self::Folder { name, .. } => name,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroLibrary {
    pub root_path: PathBuf,
    pub tree: Vec<MacroNode>,
}

pub fn init() -> MacroLibrary {
    let proj_dirs = ProjectDirs::from("com", "Nexsq", "nuui")
        .expect("Failed to locate the system configuration directory.");

    let lib_dir = proj_dirs.config_dir().join("lib");

    if !lib_dir.exists() {
        if let Err(e) = fs::create_dir_all(&lib_dir) {
            panic!(
                "Failed to create library directory at {:?}\nDetails: {}",
                lib_dir, e
            );
        }
    }

    let tree = scan_lib(&lib_dir);

    MacroLibrary {
        root_path: lib_dir,
        tree,
    }
}

fn scan_lib(path: &Path) -> Vec<MacroNode> {
    let mut nodes = Vec::new();

    if let Ok(entries) = fs::read_dir(path) {
        for entry in entries.filter_map(Result::ok) {
            if let Ok(file_type) = entry.file_type() {
                let file_name = entry.file_name();
                let name_str = file_name.to_string_lossy();
                if name_str.starts_with('.') {
                    continue;
                }

                let raw_name = name_str.into_owned();
                let entry_path = entry.path();

                if file_type.is_dir() {
                    nodes.push(MacroNode::Folder {
                        name: raw_name,
                        children: scan_lib(&entry_path),
                        path: entry_path,
                    });
                } else if file_type.is_file() {
                    if entry_path.extension().map_or(false, |ext| ext == "nuui") {
                        let clean_name = entry_path
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or(raw_name);

                        nodes.push(MacroNode::Script {
                            name: clean_name,
                            path: entry_path,
                        });
                    }
                }
            }
        }
    }

    nodes.sort_unstable_by(|a, b| match (a, b) {
        (MacroNode::Folder { .. }, MacroNode::Script { .. }) => Ordering::Less,
        (MacroNode::Script { .. }, MacroNode::Folder { .. }) => Ordering::Greater,
        _ => a.name().cmp(b.name()),
    });

    nodes
}
