//! Shared compositor policy that can be tested without a live Wayland server.

pub const DEFAULT_OUTPUT_W: i32 = 1024;
pub const DEFAULT_OUTPUT_H: i32 = 768;
pub const DEFAULT_WINDOW_W: i32 = 640;
pub const DEFAULT_WINDOW_H: i32 = 480;
pub const INITIAL_WINDOW_X: i32 = 64;
pub const INITIAL_WINDOW_Y: i32 = 64;
pub const CASCADE_STEP: i32 = 32;
pub const CASCADE_WRAP: i32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputConfig {
    pub width: i32,
    pub height: i32,
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self {
            width: DEFAULT_OUTPUT_W,
            height: DEFAULT_OUTPUT_H,
        }
    }
}

impl OutputConfig {
    pub fn from_env() -> Self {
        Self::from_env_values(
            std::env::var("RETROSHELL_COMPOSITOR_WIDTH").ok(),
            std::env::var("RETROSHELL_COMPOSITOR_HEIGHT").ok(),
        )
    }

    pub fn from_env_values(width: Option<String>, height: Option<String>) -> Self {
        Self {
            width: parse_positive_i32(width).unwrap_or(DEFAULT_OUTPUT_W),
            height: parse_positive_i32(height).unwrap_or(DEFAULT_OUTPUT_H),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowGeometry {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl WindowGeometry {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn contains_f64(self, x: f64, y: f64) -> bool {
        let x = x as i32;
        let y = y as i32;
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }
}

pub fn cascade_position(offset: i32) -> (i32, i32) {
    (INITIAL_WINDOW_X + offset, INITIAL_WINDOW_Y + offset)
}

pub fn next_cascade_offset(offset: i32) -> i32 {
    (offset + CASCADE_STEP) % CASCADE_WRAP
}

pub fn topmost_window_at(windows: &[WindowGeometry], x: f64, y: f64) -> Option<usize> {
    windows
        .iter()
        .enumerate()
        .rev()
        .find(|(_, window)| window.contains_f64(x, y))
        .map(|(idx, _)| idx)
}

pub fn move_to_top<T>(windows: &mut Vec<T>, idx: usize) {
    let window = windows.remove(idx);
    windows.push(window);
}

fn parse_positive_i32(value: Option<String>) -> Option<i32> {
    value?.parse::<i32>().ok().filter(|value| *value > 0)
}
