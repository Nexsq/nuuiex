use crate::conf::{Config, ConfigVar};
use crate::{Border, Box, Canvas, Color, Modifier, Style};

pub struct MainView {
    pub min_w: u16,
    pub min_h: u16,
    main_box: Box,
    main_x: i16,
    main_y: i16,
    tabs_box: Box,
    tabs_x: i16,
    tabs_y: i16,
    title_box: Box,
    title_x: i16,
    title_y: i16,
}

impl MainView {
    pub fn new(config: &Config) -> Self {
        let min_w = match config.vars.get("min_w").unwrap() {
            ConfigVar::Int(w) => *w as u16,
            _ => unreachable!(),
        };
        let min_h = match config.vars.get("min_h").unwrap() {
            ConfigVar::Int(h) => *h as u16,
            _ => unreachable!(),
        };
        let title_text = match config.vars.get("title_s").unwrap() {
            ConfigVar::Text(s) => s.clone(),
            _ => unreachable!(),
        };

        let main_cfg = config.boxes.iter().find(|b| b.name == "main").unwrap();
        let tabs_cfg = config.boxes.iter().find(|b| b.name == "tabs").unwrap();
        let title_cfg = config.boxes.iter().find(|b| b.name == "title").unwrap();

        let main_box = Box::new(
            main_cfg.width,
            main_cfg.height,
            main_cfg.padding,
            main_cfg.border,
            main_cfg.style,
        );
        let tabs_box = Box::new(
            tabs_cfg.width,
            tabs_cfg.height,
            tabs_cfg.padding,
            tabs_cfg.border,
            tabs_cfg.style,
        );
        let mut title_box = Box::new(
            title_cfg.width,
            title_cfg.height,
            title_cfg.padding,
            title_cfg.border,
            title_cfg.style,
        );

        let mut text_box = Box::new(
            title_cfg.width,
            title_cfg.height,
            0,
            Border::None,
            Style::default(),
        );
        let text_style = Style {
            fg: Color::Red,
            bg: Color::None,
            md: Modifier::Bold,
        };

        text_box.insert_text(&title_text, 0, 0, false, text_style);

        title_box.insert_box(
            &text_box,
            title_cfg.padding as i16,
            title_cfg.padding as i16,
        );

        Self {
            min_w,
            min_h,
            main_box,
            main_x: main_cfg.x,
            main_y: main_cfg.y,
            tabs_box,
            tabs_x: tabs_cfg.x,
            tabs_y: tabs_cfg.y,
            title_box,
            title_x: title_cfg.x,
            title_y: title_cfg.y,
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas.put_box(&self.main_box, self.main_x, self.main_y);
        canvas.put_box(&self.tabs_box, self.tabs_x, self.tabs_y);
        canvas.put_box(&self.title_box, self.title_x, self.title_y);
    }
}
