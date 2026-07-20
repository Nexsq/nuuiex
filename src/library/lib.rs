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

    pub fn path(&self) -> &Path {
        match self {
            Self::Script { path, .. } => path,
            Self::Folder { path, .. } => path,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MacroLibrary {
    pub root_path: PathBuf,
    pub tree: Vec<MacroNode>,
}

pub fn get_order_path() -> Result<PathBuf, String> {
    let config_dir = crate::get_config_dir()?;
    Ok(config_dir.join("conf").join("order.conf"))
}

pub fn load_custom_order() -> Vec<String> {
    if let Ok(path) = get_order_path() {
        if let Ok(contents) = fs::read_to_string(path) {
            return contents.lines().map(|s| s.to_string()).collect();
        }
    }
    Vec::new()
}

pub fn save_custom_order(order: &[String]) {
    if let Ok(path) = get_order_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(path, order.join("\n"));
    }
}

pub fn reset_custom_order() {
    if let Ok(path) = get_order_path() {
        let _ = fs::remove_file(path);
    }
}

pub fn reset_library() {
    if let Ok(config_dir) = crate::get_config_dir() {
        let lib_dir = config_dir.join("lib");
        if lib_dir.exists() {
            let _ = fs::remove_dir_all(&lib_dir);
            let _ = fs::create_dir_all(&lib_dir);
        }
        let md_dir = config_dir.join("macrodata");
        if md_dir.exists() {
            let _ = fs::remove_dir_all(&md_dir);
            let _ = fs::create_dir_all(&md_dir);
        }
    }
}

pub fn reset_macrodata() {
    if let Ok(config_dir) = crate::get_config_dir() {
        let md_dir = config_dir.join("macrodata");
        if md_dir.exists() {
            let _ = std::fs::remove_dir_all(&md_dir);
            let _ = std::fs::create_dir_all(&md_dir);
        }
    }
}

pub fn rename_macrodata(old_path: &Path, new_path: &Path) {
    if let Ok(config_dir) = crate::get_config_dir() {
        let lib_dir = config_dir.join("lib");
        let md_dir = config_dir.join("macrodata");

        if let (Ok(old_rel), Ok(new_rel)) = (
            old_path.strip_prefix(&lib_dir),
            new_path.strip_prefix(&lib_dir),
        ) {
            let mut old_md = md_dir.join(old_rel);
            let mut new_md = md_dir.join(new_rel);

            let is_file = old_path.extension().map_or(false, |ext| ext == "nuui");
            if is_file {
                old_md.set_extension("nuuidata");
                new_md.set_extension("nuuidata");
            }

            if old_md.exists() {
                if let Some(parent) = new_md.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::rename(old_md, new_md);
            }
        }
    }
}

pub fn delete_macrodata(path: &Path, is_folder: bool) {
    if let Ok(config_dir) = crate::get_config_dir() {
        let lib_dir = config_dir.join("lib");
        let md_dir = config_dir.join("macrodata");

        if let Ok(rel) = path.strip_prefix(&lib_dir) {
            let mut md_path = md_dir.join(rel);
            if is_folder {
                if md_path.exists() {
                    let _ = std::fs::remove_dir_all(&md_path);
                }
            } else {
                md_path.set_extension("nuuidata");
                if md_path.exists() {
                    let _ = std::fs::remove_file(&md_path);
                }
            }
        }
    }
}

pub fn init(sorting: &str) -> Result<MacroLibrary, String> {
    let config_dir = crate::get_config_dir()?;
    let lib_dir: PathBuf = config_dir.join("lib");
    let md_dir: PathBuf = config_dir.join("macrodata");

    if !lib_dir.exists() {
        if let Err(e) = fs::create_dir_all(&lib_dir) {
            return Err(format!(
                "Failed to create library directory at {:?}\nDetails: {}",
                lib_dir, e
            ));
        }
    }

    if !md_dir.exists() {
        let _ = fs::create_dir_all(&md_dir);
    }

    let custom_order = if sorting == "custom" {
        load_custom_order()
    } else {
        Vec::new()
    };

    let mut order_map = std::collections::HashMap::with_capacity(custom_order.len());
    for (i, p) in custom_order.iter().enumerate() {
        order_map.insert(p.clone(), i);
    }

    let tree = scan_lib(&lib_dir, &lib_dir, sorting, &order_map, 0);

    Ok(MacroLibrary {
        root_path: lib_dir.clone(),
        tree,
    })
}

fn scan_lib(
    path: &Path,
    root_path: &Path,
    sorting: &str,
    order_map: &std::collections::HashMap<String, usize>,
    depth: usize,
) -> Vec<MacroNode> {
    let mut nodes = Vec::new();

    if depth > 32 {
        return nodes;
    }

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
                        children: scan_lib(&entry_path, root_path, sorting, order_map, depth + 1),
                        path: entry_path.clone(),
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

    nodes.sort_unstable_by(|a, b| {
        if sorting == "custom" {
            let path_a_str = a
                .path()
                .strip_prefix(root_path)
                .unwrap_or(a.path())
                .to_str()
                .unwrap_or("");
            let path_b_str = b
                .path()
                .strip_prefix(root_path)
                .unwrap_or(b.path())
                .to_str()
                .unwrap_or("");

            let idx_a = order_map.get(path_a_str).copied().unwrap_or(usize::MAX);
            let idx_b = order_map.get(path_b_str).copied().unwrap_or(usize::MAX);

            if idx_a != idx_b {
                return idx_a.cmp(&idx_b);
            }
        }

        let folder_a = matches!(a, MacroNode::Folder { .. });
        let folder_b = matches!(b, MacroNode::Folder { .. });

        match (folder_a, folder_b) {
            (true, false) => Ordering::Less,
            (false, true) => Ordering::Greater,
            _ => {
                if sorting == "descending" {
                    b.name().cmp(a.name())
                } else {
                    a.name().cmp(b.name())
                }
            }
        }
    });

    nodes
}
