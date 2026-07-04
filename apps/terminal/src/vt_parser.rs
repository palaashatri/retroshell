use crate::terminal::Terminal;
use vte::{Params, Perform};

pub struct VtHandler<'a> {
    pub term: &'a mut Terminal,
}

impl<'a> Perform for VtHandler<'a> {
    fn print(&mut self, c: char) {
        self.term.print_char(c);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            0x0a => {
                // LF
                self.term.cursor_y += 1;
                if self.term.cursor_y >= self.term.rows {
                    self.term.scroll_up();
                    self.term.cursor_y = self.term.rows - 1;
                }
            }
            0x0d => {
                // CR
                self.term.cursor_x = 0;
            }
            0x08 if self.term.cursor_x > 0 => {
                // BS
                self.term.cursor_x -= 1;
            }
            _ => {}
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _action: char) {}
    fn put(&mut self, _byte: u8) {}
    fn unhook(&mut self) {}
    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(
        &mut self,
        params: &Params,
        _intermediates: &[u8],
        _ignore: bool,
        action: char,
    ) {
        match action {
            'm' => {
                let flat: Vec<u16> = params.iter().flat_map(|p| p.iter()).copied().collect();
                let mut i = 0;
                while i < flat.len() {
                    match flat[i] {
                        0 => {
                            self.term.current_fg = retro_kit::Color::WHITE;
                            self.term.current_bg = retro_kit::Color::BLACK;
                            self.term.current_bold = false;
                            self.term.current_italic = false;
                            self.term.current_underline = false;
                        }
                        1 => {
                            self.term.current_bold = true;
                        }
                        3 => {
                            self.term.current_italic = true;
                        }
                        4 => {
                            self.term.current_underline = true;
                        }
                        30..=37 => {
                            self.term.current_fg = map_ansi_color(flat[i] - 30);
                        }
                        38 => {
                            if i + 2 < flat.len() && flat[i + 1] == 5 {
                                self.term.current_fg = map_256_color(flat[i + 2]);
                                i += 2;
                            } else if i + 4 < flat.len() && flat[i + 1] == 2 {
                                let r = flat[i + 2] as f32 / 255.0;
                                let g = flat[i + 3] as f32 / 255.0;
                                let b = flat[i + 4] as f32 / 255.0;
                                self.term.current_fg = retro_kit::Color::new(r, g, b, 1.0);
                                i += 4;
                            }
                        }
                        40..=47 => {
                            self.term.current_bg = map_ansi_color(flat[i] - 40);
                        }
                        48 => {
                            if i + 2 < flat.len() && flat[i + 1] == 5 {
                                self.term.current_bg = map_256_color(flat[i + 2]);
                                i += 2;
                            } else if i + 4 < flat.len() && flat[i + 1] == 2 {
                                let r = flat[i + 2] as f32 / 255.0;
                                let g = flat[i + 3] as f32 / 255.0;
                                let b = flat[i + 4] as f32 / 255.0;
                                self.term.current_bg = retro_kit::Color::new(r, g, b, 1.0);
                                i += 4;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
            }
            'H' | 'f' => {
                let mut row = 1;
                let mut col = 1;
                let mut iter = params.iter();
                if let Some(r_param) = iter.next() {
                    if let Some(r) = r_param.first() {
                        row = *r;
                    }
                }
                if let Some(c_param) = iter.next() {
                    if let Some(c) = c_param.first() {
                        col = *c;
                    }
                }
                self.term.cursor_y = (row as usize).saturating_sub(1).min(self.term.rows - 1);
                self.term.cursor_x = (col as usize).saturating_sub(1).min(self.term.cols - 1);
            }
            'J' => {
                let mut val = 0;
                if let Some(p) = params.iter().next().and_then(|p| p.first()) {
                    val = *p;
                }
                if val == 2 {
                    self.term.grid.fill(crate::terminal::Cell::default());
                    self.term.cursor_x = 0;
                    self.term.cursor_y = 0;
                }
            }
            'K' => {
                let mut mode = 0;
                if let Some(p) = params.iter().next().and_then(|p| p.first()) {
                    mode = *p;
                }
                let row = self.term.cursor_y;
                let cols = self.term.cols;
                match mode {
                    0 => {
                        for col in self.term.cursor_x..cols {
                            let idx = row * cols + col;
                            if idx < self.term.grid.len() {
                                self.term.grid[idx] = crate::terminal::Cell::default();
                            }
                        }
                    }
                    1 => {
                        for col in 0..=self.term.cursor_x.min(cols - 1) {
                            let idx = row * cols + col;
                            if idx < self.term.grid.len() {
                                self.term.grid[idx] = crate::terminal::Cell::default();
                            }
                        }
                    }
                    2 => {
                        for col in 0..cols {
                            let idx = row * cols + col;
                            if idx < self.term.grid.len() {
                                self.term.grid[idx] = crate::terminal::Cell::default();
                            }
                        }
                    }
                    _ => {}
                }
            }
            'r' => {
                let mut top: usize = 1;
                let mut bottom: usize = self.term.rows;
                let mut iter = params.iter();
                if let Some(t_param) = iter.next() {
                    if let Some(t) = t_param.first() {
                        top = *t as usize;
                    }
                }
                if let Some(b_param) = iter.next() {
                    if let Some(b) = b_param.first() {
                        bottom = *b as usize;
                    }
                }
                self.term.scroll_top = (top as usize).saturating_sub(1).min(self.term.rows - 1);
                self.term.scroll_bottom = (bottom as usize).saturating_sub(1).min(self.term.rows - 1);
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, _byte: u8) {}
}

fn map_ansi_color(code: u16) -> retro_kit::Color {
    match code {
        0 => retro_kit::Color::BLACK,
        1 => retro_kit::Color::new(0.8, 0.0, 0.0, 1.0),
        2 => retro_kit::Color::new(0.0, 0.8, 0.0, 1.0),
        3 => retro_kit::Color::new(0.8, 0.8, 0.0, 1.0),
        4 => retro_kit::Color::new(0.0, 0.0, 0.8, 1.0),
        5 => retro_kit::Color::new(0.8, 0.0, 0.8, 1.0),
        6 => retro_kit::Color::new(0.0, 0.8, 0.8, 1.0),
        _ => retro_kit::Color::WHITE,
    }
}

fn map_256_color(idx: u16) -> retro_kit::Color {
    if idx < 8 {
        map_ansi_color(idx)
    } else if idx < 16 {
        match idx {
            8 => retro_kit::Color::new(0.3, 0.3, 0.3, 1.0),
            9 => retro_kit::Color::new(1.0, 0.3, 0.3, 1.0),
            10 => retro_kit::Color::new(0.3, 1.0, 0.3, 1.0),
            11 => retro_kit::Color::new(1.0, 1.0, 0.3, 1.0),
            12 => retro_kit::Color::new(0.3, 0.3, 1.0, 1.0),
            13 => retro_kit::Color::new(1.0, 0.3, 1.0, 1.0),
            14 => retro_kit::Color::new(0.3, 1.0, 1.0, 1.0),
            _ => retro_kit::Color::WHITE,
        }
    } else if idx < 232 {
        let cube_idx = idx - 16;
        let r = (cube_idx / 36) % 6;
        let g = (cube_idx / 6) % 6;
        let b = cube_idx % 6;
        retro_kit::Color::new(
            r as f32 / 5.0,
            g as f32 / 5.0,
            b as f32 / 5.0,
            1.0,
        )
    } else {
        let gray = (idx - 232) as f32 / 23.0;
        retro_kit::Color::new(gray, gray, gray, 1.0)
    }
}
