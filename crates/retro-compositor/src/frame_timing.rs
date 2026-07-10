//! VRR (Variable Refresh Rate) and frame timing support.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Supported refresh rates (Hz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum RefreshRate {
    Hz60 = 60,
    Hz120 = 120,
    Hz144 = 144,
    Hz165 = 165,
    Adaptive = 0, // VRR (variable)
}

impl RefreshRate {
    pub fn as_hz(&self) -> u32 {
        *self as u32
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Hz60 => "60hz",
            Self::Hz120 => "120hz",
            Self::Hz144 => "144hz",
            Self::Hz165 => "165hz",
            Self::Adaptive => "adaptive",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Self::parse_flexible(s)
    }

    /// Parse refresh rate from settings/env (`60`, `60hz`, `120Hz`, `adaptive`).
    pub fn parse_flexible(s: &str) -> Option<Self> {
        let s = s.trim().to_ascii_lowercase();
        match s.as_str() {
            "60" | "60hz" | "60h" => Some(Self::Hz60),
            "120" | "120hz" => Some(Self::Hz120),
            "144" | "144hz" => Some(Self::Hz144),
            "165" | "165hz" => Some(Self::Hz165),
            "adaptive" | "vrr" | "variable" | "0" => Some(Self::Adaptive),
            _ => None,
        }
    }

    pub fn frame_duration(&self) -> Duration {
        match self {
            Self::Hz60 => Duration::from_nanos(1_000_000_000 / 60),
            Self::Hz120 => Duration::from_nanos(1_000_000_000 / 120),
            Self::Hz144 => Duration::from_nanos(1_000_000_000 / 144),
            Self::Hz165 => Duration::from_nanos(1_000_000_000 / 165),
            // Adaptive: short poll so X11 Present / damage can drive timing.
            Self::Adaptive => Duration::from_millis(1),
        }
    }

    /// Whether this rate means "pace with FrameScheduler" (false for Adaptive/VRR).
    pub fn is_fixed(&self) -> bool {
        !matches!(self, Self::Adaptive)
    }
}

/// Frame timing and VRR scheduler.
pub struct FrameScheduler {
    target_refresh_rate: RefreshRate,
    last_frame_time: Option<Instant>,
    frame_times: VecDeque<Duration>,
    max_frame_history: usize,
}

impl Default for FrameScheduler {
    fn default() -> Self {
        Self::new(RefreshRate::Hz60)
    }
}

impl FrameScheduler {
    pub fn new(target_refresh_rate: RefreshRate) -> Self {
        Self {
            target_refresh_rate,
            last_frame_time: None,
            frame_times: VecDeque::new(),
            max_frame_history: 120, // Track last 120 frames
        }
    }

    /// Set the target refresh rate.
    pub fn set_refresh_rate(&mut self, rate: RefreshRate) {
        self.target_refresh_rate = rate;
    }

    /// Get the target refresh rate.
    pub fn refresh_rate(&self) -> RefreshRate {
        self.target_refresh_rate
    }

    /// Record a frame render time. Returns whether we should wait before the next frame.
    pub fn record_frame(&mut self) -> bool {
        let now = Instant::now();

        if let Some(last_time) = self.last_frame_time {
            let elapsed = now.duration_since(last_time);
            self.frame_times.push_back(elapsed);

            if self.frame_times.len() > self.max_frame_history {
                self.frame_times.pop_front();
            }
        }

        self.last_frame_time = Some(now);
        true
    }

    /// Calculate the time to wait before presenting the next frame.
    /// This implements VSync-synchronized frame pacing.
    pub fn time_until_next_frame(&self) -> Duration {
        if let Some(last_time) = self.last_frame_time {
            let target_duration = self.target_refresh_rate.frame_duration();
            let elapsed = last_time.elapsed();

            if elapsed < target_duration {
                target_duration - elapsed
            } else {
                Duration::ZERO
            }
        } else {
            Duration::ZERO
        }
    }

    /// Get the average frame time over recent history.
    pub fn average_frame_time(&self) -> Duration {
        if self.frame_times.is_empty() {
            return Duration::ZERO;
        }

        let total: Duration = self.frame_times.iter().sum();
        total / self.frame_times.len() as u32
    }

    /// Get the current FPS based on recent frame times.
    pub fn current_fps(&self) -> f32 {
        let avg = self.average_frame_time();
        if avg.as_secs_f32() > 0.0 {
            1.0 / avg.as_secs_f32()
        } else {
            0.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_refresh_rate_duration() {
        assert!(RefreshRate::Hz60.frame_duration().as_millis() <= 17);
        assert!(RefreshRate::Hz120.frame_duration().as_millis() <= 9);
        assert!(RefreshRate::Hz144.frame_duration().as_millis() <= 7);
        assert!(RefreshRate::Adaptive.is_fixed() == false);
        assert!(RefreshRate::Hz60.is_fixed());
    }

    #[test]
    fn test_refresh_rate_serialize() {
        assert_eq!(RefreshRate::Hz60.as_str(), "60hz");
        assert_eq!(RefreshRate::Hz120.as_str(), "120hz");
        assert_eq!(RefreshRate::Adaptive.as_str(), "adaptive");
    }

    #[test]
    fn test_refresh_rate_deserialize() {
        assert_eq!(RefreshRate::from_str("60hz"), Some(RefreshRate::Hz60));
        assert_eq!(RefreshRate::from_str("120hz"), Some(RefreshRate::Hz120));
        assert_eq!(RefreshRate::from_str("adaptive"), Some(RefreshRate::Adaptive));
        assert_eq!(RefreshRate::from_str("invalid"), None);
        assert_eq!(RefreshRate::parse_flexible("60"), Some(RefreshRate::Hz60));
        assert_eq!(RefreshRate::parse_flexible("VRR"), Some(RefreshRate::Adaptive));
    }

    #[test]
    fn test_frame_scheduler_timing() {
        let mut scheduler = FrameScheduler::new(RefreshRate::Hz60);
        scheduler.record_frame();
        std::thread::sleep(Duration::from_millis(20));
        let wait_time = scheduler.time_until_next_frame();
        // Should be <= 16ms since we already waited 20ms
        assert!(wait_time.as_millis() <= 16);
    }

    #[test]
    fn test_frame_scheduler_fps() {
        let mut scheduler = FrameScheduler::new(RefreshRate::Hz60);
        for _ in 0..10 {
            scheduler.record_frame();
            std::thread::sleep(Duration::from_millis(16));
        }
        let fps = scheduler.current_fps();
        // Should be approximately 60 FPS
        assert!(fps > 50.0 && fps < 70.0);
    }
}
