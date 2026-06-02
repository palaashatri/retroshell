#![allow(dead_code)]

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
                for param in params {
                    for val in param {
                        match val {
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
                                self.term.current_fg = map_ansi_color(*val - 30);
                            }
                            40..=47 => {
                                self.term.current_bg = map_ansi_color(*val - 40);
                            }
                            _ => {}
                        }
                    }
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
