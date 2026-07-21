use super::layout::TABS_W;
use super::{ActivePanel, ListInputMode};
use crate::{
    Box, Color, Gradient, Modifier, conf::Config, lib::MacroNode, theme::themecore::Theme,
};

pub fn resolve_view(
    expanded_path: &[usize],
    library_tree: &[MacroNode],
    is_creating: bool,
) -> (Vec<usize>, Option<usize>) {
    let mut view_path = Vec::with_capacity(expanded_path.len());
    let mut current_nodes = library_tree;
    for &idx in expanded_path {
        if let Some(MacroNode::Folder { children, .. }) = current_nodes.get(idx) {
            view_path.push(idx);
            current_nodes = children;
        } else {
            break;
        }
    }

    if !view_path.is_empty() && current_nodes.is_empty() && !is_creating {
        let empty_idx = view_path.pop();
        (view_path, empty_idx)
    } else {
        (view_path, None)
    }
}

pub fn refresh(
    term_h: u16,
    header_h: u16,
    active: ActivePanel,
    library_tree: &[MacroNode],
    expanded_path: &[usize],
    list_selected: &mut usize,
    list_scroll: &mut usize,
    editing_path: Option<&std::path::Path>,
    is_dirty: bool,
    config: &Config,
    theme: &Theme,
    list_input: &ListInputMode,
) -> Box {
    let list_h = term_h.saturating_sub(header_h);
    let is_active = active == ActivePanel::List;
    let use_border_color = config.indicator_style == "border";

    let list_color = if is_active && use_border_color {
        &theme.selected_box
    } else {
        &theme.list_box
    };

    let list_w = if config.tabs_num == 1 {
        config.lib_width as u16 + TABS_W - 1
    } else {
        config.lib_width as u16
    };

    let mut list_box = Box::new(
        list_w,
        list_h,
        1,
        config.get_border(),
        list_color.clone(),
        Gradient::Solid(Color::None),
        Modifier::None,
    );

    crate::panels::apply_indicator(&mut list_box, config, theme, is_active);

    let is_creating = matches!(
        list_input,
        ListInputMode::CreatingFile(_) | ListInputMode::CreatingFolder(_)
    );
    let (view_path, empty_child_idx) = resolve_view(expanded_path, library_tree, is_creating);

    let mut parent_nodes = library_tree;
    if !view_path.is_empty() {
        for &idx in &view_path[..view_path.len() - 1] {
            if let Some(MacroNode::Folder { children, .. }) = parent_nodes.get(idx) {
                parent_nodes = children;
            }
        }
    }

    struct RenderItem<'a> {
        node: Option<&'a MacroNode>,
        custom_text: Option<&'a str>,
        indent_spaces: usize,
        use_branch_prefix: bool,
        is_selectable: bool,
        selectable_idx: Option<usize>,
        is_active_parent: bool,
        is_input: bool,
        input_is_folder: bool,
        is_empty_indicator: bool,
    }

    let mut render_items: Vec<RenderItem> = Vec::new();

    let push_empty_indicator = |items: &mut Vec<RenderItem>, indent: usize, branch: bool| {
        items.push(RenderItem {
            node: None,
            custom_text: Some("..."),
            indent_spaces: indent,
            use_branch_prefix: branch,
            is_selectable: false,
            selectable_idx: None,
            is_active_parent: false,
            is_input: false,
            input_is_folder: false,
            is_empty_indicator: true,
        });
    };

    if view_path.is_empty() {
        if parent_nodes.is_empty() && *list_input == ListInputMode::None {
            push_empty_indicator(&mut render_items, 0, false);
        } else {
            for (i, node) in parent_nodes.iter().enumerate() {
                let is_renaming =
                    matches!(list_input, ListInputMode::Renaming(_)) && *list_selected == i;
                let custom_text = if is_renaming {
                    if let ListInputMode::Renaming(n) = list_input {
                        Some(n.as_str())
                    } else {
                        None
                    }
                } else {
                    None
                };

                render_items.push(RenderItem {
                    node: Some(node),
                    custom_text,
                    indent_spaces: 0,
                    use_branch_prefix: false,
                    is_selectable: true,
                    selectable_idx: Some(i),
                    is_active_parent: false,
                    is_input: is_renaming,
                    input_is_folder: false,
                    is_empty_indicator: false,
                });

                if empty_child_idx == Some(i) && *list_input == ListInputMode::None {
                    push_empty_indicator(&mut render_items, 1, true);
                }
            }

            match list_input {
                ListInputMode::CreatingFile(n) | ListInputMode::CreatingFolder(n) => {
                    render_items.push(RenderItem {
                        node: None,
                        custom_text: Some(n.as_str()),
                        indent_spaces: 0,
                        use_branch_prefix: false,
                        is_selectable: false,
                        selectable_idx: None,
                        is_active_parent: false,
                        is_input: true,
                        input_is_folder: matches!(list_input, ListInputMode::CreatingFolder(_)),
                        is_empty_indicator: false,
                    });
                }
                _ => {}
            }
        }
    } else {
        let active_folder_idx = *view_path.last().unwrap();

        for (idx, node) in parent_nodes.iter().enumerate() {
            if idx != active_folder_idx {
                render_items.push(RenderItem {
                    node: Some(node),
                    custom_text: None,
                    indent_spaces: 0,
                    use_branch_prefix: false,
                    is_selectable: false,
                    selectable_idx: None,
                    is_active_parent: false,
                    is_input: false,
                    input_is_folder: false,
                    is_empty_indicator: false,
                });
            } else {
                render_items.push(RenderItem {
                    node: Some(node),
                    custom_text: None,
                    indent_spaces: 0,
                    use_branch_prefix: false,
                    is_selectable: false,
                    selectable_idx: None,
                    is_active_parent: true,
                    is_input: false,
                    input_is_folder: false,
                    is_empty_indicator: false,
                });

                if let MacroNode::Folder { children, .. } = node {
                    if children.is_empty() && *list_input == ListInputMode::None {
                        push_empty_indicator(&mut render_items, 1, true);
                    } else {
                        for (child_idx, child_node) in children.iter().enumerate() {
                            let is_renaming = matches!(list_input, ListInputMode::Renaming(_))
                                && *list_selected == child_idx;
                            let custom_text = if is_renaming {
                                if let ListInputMode::Renaming(n) = list_input {
                                    Some(n.as_str())
                                } else {
                                    None
                                }
                            } else {
                                None
                            };

                            render_items.push(RenderItem {
                                node: Some(child_node),
                                custom_text,
                                indent_spaces: 2,
                                use_branch_prefix: child_idx == 0,
                                is_selectable: true,
                                selectable_idx: Some(child_idx),
                                is_active_parent: false,
                                is_input: is_renaming,
                                input_is_folder: false,
                                is_empty_indicator: false,
                            });

                            if empty_child_idx == Some(child_idx)
                                && *list_input == ListInputMode::None
                            {
                                push_empty_indicator(&mut render_items, 3, true);
                            }
                        }
                    }

                    match list_input {
                        ListInputMode::CreatingFile(n) | ListInputMode::CreatingFolder(n) => {
                            render_items.push(RenderItem {
                                node: None,
                                custom_text: Some(n.as_str()),
                                indent_spaces: 2,
                                use_branch_prefix: children.is_empty(),
                                is_selectable: false,
                                selectable_idx: None,
                                is_active_parent: false,
                                is_input: true,
                                input_is_folder: matches!(
                                    list_input,
                                    ListInputMode::CreatingFolder(_)
                                ),
                                is_empty_indicator: false,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    let max_selectable_idx = if view_path.is_empty() {
        parent_nodes.len().saturating_sub(1)
    } else {
        let active_folder_idx = *view_path.last().unwrap();
        if let Some(MacroNode::Folder { children, .. }) = parent_nodes.get(active_folder_idx) {
            children.len().saturating_sub(1)
        } else {
            0
        }
    };
    *list_selected = (*list_selected).min(max_selectable_idx);

    let target_render_idx = if matches!(
        list_input,
        ListInputMode::CreatingFile(_) | ListInputMode::CreatingFolder(_)
    ) {
        render_items
            .iter()
            .position(|item| item.is_input)
            .unwrap_or(0)
    } else {
        render_items
            .iter()
            .position(|item| item.is_selectable && item.selectable_idx == Some(*list_selected))
            .unwrap_or(0)
    };

    let visible_items = list_h.saturating_sub(2) as usize;
    if target_render_idx < *list_scroll {
        *list_scroll = target_render_idx;
    } else if target_render_idx >= *list_scroll + visible_items && visible_items > 0 {
        *list_scroll = target_render_idx.saturating_sub(visible_items - 1);
    }

    let max_text_len = list_w.saturating_sub(2) as usize;

    for (display_line, item) in render_items
        .iter()
        .enumerate()
        .skip(*list_scroll)
        .take(visible_items)
    {
        let (node_symbol, normal_color) = if let Some(n) = item.node {
            match n {
                MacroNode::Folder { .. } => ("▪", &theme.list_folder),
                MacroNode::Script { .. } => ("▫", &theme.list_file),
            }
        } else if item.is_input {
            if item.input_is_folder {
                ("▪", &theme.list_folder)
            } else {
                ("▫", &theme.list_file)
            }
        } else {
            ("", &theme.settings_entry)
        };

        let is_selected = item.is_selectable
            && (item.selectable_idx == Some(*list_selected))
            && is_active
            && *list_input == ListInputMode::None;

        let is_empty_indicator = item.is_empty_indicator;
        let use_dim =
            !item.is_selectable && !is_empty_indicator && !item.is_active_parent && !item.is_input;

        let (fg_color, bg_color) = if item.is_active_parent {
            (normal_color.clone(), Gradient::Solid(Color::None))
        } else if item.is_input {
            (Gradient::Solid(Color::Black), normal_color.clone())
        } else if !item.is_selectable && !is_empty_indicator {
            (theme.settings_entry.clone(), Gradient::Solid(Color::None))
        } else if is_selected {
            (Gradient::Solid(Color::Black), normal_color.clone())
        } else {
            (normal_color.clone(), Gradient::Solid(Color::None))
        };

        let md = if is_selected || item.is_input {
            Modifier::None
        } else if use_dim {
            Modifier::Dim
        } else {
            Modifier::None
        };

        let mut prefix = String::with_capacity(item.indent_spaces + 4);
        match (item.indent_spaces, item.use_branch_prefix) {
            (1, true) => prefix.push('┗'),
            (2, true) => prefix.push_str("┗ "),
            (n, true) => {
                for _ in 0..n.saturating_sub(1) {
                    prefix.push(' ');
                }
                prefix.push('┗');
            }
            (n, false) => {
                for _ in 0..n {
                    prefix.push(' ');
                }
            }
        }
        prefix.push_str(node_symbol);

        let mut base_name = if item.is_input {
            if let Some(custom) = item.custom_text {
                custom.to_string()
            } else {
                String::new()
            }
        } else if let Some(n) = item.node {
            n.name().to_string()
        } else if let Some(custom) = item.custom_text {
            custom.to_string()
        } else {
            String::new()
        };

        if item.is_input {
            base_name.push('_');
        }

        let is_editing_this = if let Some(n) = item.node {
            Some(n.path()) == editing_path
        } else {
            false
        };

        if !base_name.is_empty() && !prefix.is_empty() {
            prefix.push(' ');
        }

        let target_len = max_text_len.saturating_sub(2);
        let e_len = if is_editing_this { 2 } else { 0 };
        let allowed_len = target_len.saturating_sub(e_len);

        let prefix_chars = prefix.chars().count();
        let allowed_name_len = allowed_len.saturating_sub(prefix_chars);
        let name_chars = base_name.chars().count();

        if name_chars > allowed_name_len {
            if item.is_input {
                let skip = name_chars.saturating_sub(allowed_name_len);
                base_name = base_name.chars().skip(skip).collect();
            } else if let Some((byte_idx, _)) = base_name.char_indices().nth(allowed_name_len) {
                base_name.truncate(byte_idx);
            }
        }

        prefix.push_str(&base_name);
        let mut text = prefix;

        let char_count = text.chars().count();
        if char_count > allowed_len {
            if let Some((byte_idx, _)) = text.char_indices().nth(allowed_len) {
                text.truncate(byte_idx);
            }
        } else {
            let padding = allowed_len.saturating_sub(char_count);
            text.reserve(padding);
            for _ in 0..padding {
                text.push(' ');
            }
        }

        if is_editing_this {
            if is_dirty {
                text.push_str(" ◦");
            } else {
                text.push_str(" •");
            }
        }

        let display_y = (display_line - *list_scroll) as i16;
        list_box.insert_text(&text, 1, display_y, false, fg_color, bg_color, md);
    }

    list_box
}
