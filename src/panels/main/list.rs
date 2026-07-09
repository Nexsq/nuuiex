use super::ActivePanel;
use super::layout::{LIST_W, TITLE_H};
use crate::{Border, Box, Color, Modifier, Style, conf::Config, lib::MacroNode};

pub fn resolve_view(
    expanded_path: &[usize],
    library_tree: &[MacroNode],
) -> (Vec<usize>, Option<usize>) {
    let mut view_path = expanded_path.to_vec();
    let mut current_nodes = library_tree;
    for &idx in &view_path {
        if let Some(MacroNode::Folder { children, .. }) = current_nodes.get(idx) {
            current_nodes = children;
        }
    }

    if !view_path.is_empty() && current_nodes.is_empty() {
        let empty_idx = view_path.pop();
        (view_path, empty_idx)
    } else {
        (view_path, None)
    }
}

pub fn refresh(
    term_h: u16,
    active: ActivePanel,
    library_tree: &[MacroNode],
    expanded_path: &[usize],
    list_selected: &mut usize,
    list_scroll: &mut usize,
    editing_path: Option<&std::path::Path>,
    config: &Config,
) -> Box {
    let list_h = term_h.saturating_sub(TITLE_H);

    let is_active = active == ActivePanel::List;
    let use_corner = config.indicator_style == "corner";

    let list_color = if is_active && !use_corner {
        Color::White
    } else {
        Color::Blue
    };
    let list_border = if is_active && !use_corner {
        Border::Heavy
    } else {
        Border::Light
    };

    let mut list_box = Box::new(
        LIST_W,
        list_h,
        1,
        list_border,
        Style {
            fg: list_color,
            bg: Color::None,
            md: Modifier::None,
        },
    );

    if is_active && use_corner {
        if LIST_W > 0 {
            list_box.put_cell(
                crate::Cell {
                    c: '■',
                    s: Style {
                        fg: Color::White,
                        bg: Color::None,
                        md: Modifier::None,
                    },
                },
                LIST_W - 1,
                0,
            );
        }
    }

    let (view_path, empty_child_idx) = resolve_view(expanded_path, library_tree);

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
        custom_text: Option<&'static str>,
        indent_spaces: usize,
        use_branch_prefix: bool,
        is_selectable: bool,
        selectable_idx: Option<usize>,
        is_active_parent: bool,
    }

    let mut render_items: Vec<RenderItem> = Vec::new();

    let push_empty_indicator = |items: &mut Vec<RenderItem>, indent: usize| {
        items.push(RenderItem {
            node: None,
            custom_text: Some("..."),
            indent_spaces: indent,
            use_branch_prefix: true,
            is_selectable: false,
            selectable_idx: None,
            is_active_parent: false,
        });
    };

    if view_path.is_empty() {
        for (i, node) in parent_nodes.iter().enumerate() {
            render_items.push(RenderItem {
                node: Some(node),
                custom_text: None,
                indent_spaces: 0,
                use_branch_prefix: false,
                is_selectable: true,
                selectable_idx: Some(i),
                is_active_parent: false,
            });

            if empty_child_idx == Some(i) {
                push_empty_indicator(&mut render_items, 1);
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
                });

                if let MacroNode::Folder { children, .. } = node {
                    if children.is_empty() {
                        push_empty_indicator(&mut render_items, 1);
                    } else {
                        for (child_idx, child_node) in children.iter().enumerate() {
                            render_items.push(RenderItem {
                                node: Some(child_node),
                                custom_text: None,
                                indent_spaces: 2,
                                use_branch_prefix: child_idx == 0,
                                is_selectable: true,
                                selectable_idx: Some(child_idx),
                                is_active_parent: false,
                            });

                            if empty_child_idx == Some(child_idx) {
                                push_empty_indicator(&mut render_items, 3);
                            }
                        }
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

    let target_render_idx = render_items
        .iter()
        .position(|item| item.is_selectable && item.selectable_idx == Some(*list_selected))
        .unwrap_or(0);

    let visible_items = list_h.saturating_sub(2) as usize;
    if target_render_idx < *list_scroll {
        *list_scroll = target_render_idx;
    } else if target_render_idx >= *list_scroll + visible_items && visible_items > 0 {
        *list_scroll = target_render_idx.saturating_sub(visible_items - 1);
    }

    let max_text_len = LIST_W.saturating_sub(2) as usize;

    for (display_line, item) in render_items
        .iter()
        .enumerate()
        .skip(*list_scroll)
        .take(visible_items)
    {
        let (node_symbol, normal_color) = if let Some(n) = item.node {
            match n {
                MacroNode::Folder { .. } => ("▪", Color::Blue),
                MacroNode::Script { .. } => ("▫", Color::Magenta),
            }
        } else {
            ("", Color::DarkGray)
        };

        let is_selected =
            item.is_selectable && (item.selectable_idx == Some(*list_selected)) && is_active;

        let (fg_color, bg_color) = if item.is_active_parent {
            (normal_color, Color::None)
        } else if !item.is_selectable {
            (Color::DarkGray, Color::None)
        } else if is_selected {
            (Color::Black, normal_color)
        } else {
            (normal_color, Color::None)
        };

        let indent_prefix = match (item.indent_spaces, item.use_branch_prefix) {
            (1, true) => "┗".to_string(),
            (2, true) => "┗ ".to_string(),
            (n, true) => format!("{}┗", " ".repeat(n.saturating_sub(1))),
            (n, false) => " ".repeat(n),
        };

        let mut text = indent_prefix;
        text.push_str(node_symbol);

        let base_name = if let Some(n) = item.node {
            n.name()
        } else if let Some(custom) = item.custom_text {
            custom
        } else {
            ""
        };

        let is_editing_this = if let Some(n) = item.node {
            Some(n.path()) == editing_path
        } else {
            false
        };

        if !base_name.is_empty() {
            if !text.is_empty() {
                text.push(' ');
            }
            text.push_str(base_name);
        }

        let char_count = text.chars().count();
        let target_len = max_text_len.saturating_sub(2);
        let e_len = if is_editing_this { 2 } else { 0 };
        let allowed_len = target_len.saturating_sub(e_len);

        if char_count > allowed_len {
            text = text.chars().take(allowed_len).collect();
        } else {
            text.push_str(&" ".repeat(allowed_len - char_count));
        }

        if is_editing_this {
            text.push_str(" •");
        }

        list_box.insert_text(
            &text,
            1,
            (display_line - *list_scroll) as i16,
            false,
            Style {
                fg: fg_color,
                bg: bg_color,
                md: Modifier::None,
            },
        );
    }

    list_box
}
