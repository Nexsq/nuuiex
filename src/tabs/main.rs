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
    pub fn new(term_w: u16, term_h: u16) -> Self {
        let min_w = 40;
        let min_h = 15;
        let title_text = String::from("NUUI");

        let main_box = Box::new(
            term_w,
            term_h.saturating_sub(6),
            0,
            Border::Light,
            Style {
                fg: Color::White,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let tabs_box = Box::new(
            term_w,
            3,
            0,
            Border::Rounded,
            Style {
                fg: Color::Cyan,
                bg: Color::None,
                md: Modifier::None,
            },
        );

        let mut title_box = Box::new(
            16,
            3,
            1,
            Border::Double,
            Style {
                fg: Color::Green,
                bg: Color::None,
                md: Modifier::Bold,
            },
        );
        let title_style = Style {
            fg: Color::Red,
            bg: Color::None,
            md: Modifier::Bold,
        };

        title_box.insert_text(&title_text, 0, 0, false, title_style);

        Self {
            min_w,
            min_h,
            main_box,
            main_x: 0,
            main_y: 6,
            tabs_box,
            tabs_x: 0,
            tabs_y: 3,
            title_box,
            title_x: 0,
            title_y: 0,
        }
    }

    pub fn render(&self, canvas: &mut Canvas) {
        canvas.put_box(&self.main_box, self.main_x, self.main_y);
        canvas.put_box(&self.tabs_box, self.tabs_x, self.tabs_y);
        canvas.put_box(&self.title_box, self.title_x, self.title_y);
    }
}
