//! HDR and color space management for the compositor.

/// Supported color spaces for output and surfaces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// Standard sRGB (SDR, 8-bit per channel)
    SRgb,
    /// Rec. 2020 (wide color gamut, 10-bit per channel)
    Rec2020,
    /// scRGB (linear, 16-bit float per channel, allows > 1.0 for HDR)
    ScRgb,
}

impl ColorSpace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::SRgb => "srgb",
            Self::Rec2020 => "rec2020",
            Self::ScRgb => "scrgb",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        Self::from_str_flexible(s)
    }

    /// Case-insensitive parse; accepts aliases used in settings/env.
    pub fn from_str_flexible(s: &str) -> Option<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "srgb" | "s-rgb" | "srgb8" => Some(Self::SRgb),
            "rec2020" | "bt2020" | "bt.2020" => Some(Self::Rec2020),
            "scrgb" | "sc-rgb" | "linear" => Some(Self::ScRgb),
            _ => None,
        }
    }
}

/// HDR capability detection and negotiation.
#[derive(Debug, Clone)]
pub struct HdrCapabilities {
    /// Whether the GPU supports HDR rendering
    pub hdr_supported: bool,
    /// Supported color spaces
    pub supported_color_spaces: Vec<ColorSpace>,
    /// Current output color space
    pub current_color_space: ColorSpace,
}

impl Default for HdrCapabilities {
    fn default() -> Self {
        Self {
            hdr_supported: false,
            supported_color_spaces: vec![ColorSpace::SRgb],
            current_color_space: ColorSpace::SRgb,
        }
    }
}

impl HdrCapabilities {
    /// Detect HDR capabilities from the GPU / display path.
    ///
    /// Nested X11 / Xvfb / software GL has no honest HDR path. Until a real
    /// DRM/KMS connector probe exists, this always reports `hdr_supported = false`.
    /// Callers may still *request* HDR via policy; [`apply_request`] refuses honestly.
    pub fn detect() -> Self {
        // Future: query DRM connector HDR static metadata / EGL colorspace
        // extensions / ten-bit fb formats. Nested X11 backend cannot claim that.
        Self {
            hdr_supported: false,
            supported_color_spaces: vec![ColorSpace::SRgb],
            current_color_space: ColorSpace::SRgb,
        }
    }

    /// Set the output color space if supported.
    pub fn set_color_space(&mut self, color_space: ColorSpace) -> bool {
        if self.supported_color_spaces.contains(&color_space) {
            self.current_color_space = color_space;
            true
        } else {
            false
        }
    }

    /// Apply a client/user policy request. Returns whether the requested color
    /// space was applied. HDR request is ignored when `hdr_supported` is false.
    pub fn apply_request(&mut self, hdr_requested: bool, color_space: ColorSpace) -> bool {
        if hdr_requested && !self.hdr_supported {
            // Keep SDR; widen supported list is not honest without hardware.
            return self.set_color_space(ColorSpace::SRgb);
        }
        if hdr_requested && self.hdr_supported {
            if !self.supported_color_spaces.contains(&ColorSpace::Rec2020) {
                self.supported_color_spaces
                    .push(ColorSpace::Rec2020);
            }
            if !self.supported_color_spaces.contains(&ColorSpace::ScRgb) {
                self.supported_color_spaces.push(ColorSpace::ScRgb);
            }
        }
        self.set_color_space(color_space)
            || self.set_color_space(ColorSpace::SRgb)
    }
}

/// Per-surface color space tracking.
#[derive(Debug, Clone)]
pub struct SurfaceColorSpace {
    /// Client-declared color space (typically sRGB for compatibility)
    pub client_color_space: ColorSpace,
    /// Output color space after tone-mapping
    pub output_color_space: ColorSpace,
}

impl Default for SurfaceColorSpace {
    fn default() -> Self {
        Self {
            client_color_space: ColorSpace::SRgb,
            output_color_space: ColorSpace::SRgb,
        }
    }
}

/// Tone-mapping mode for SDR→HDR conversion.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToneMapperMode {
    /// Reinhard tone-mapping (simple, preserves details)
    Reinhard,
    /// ACES tone-mapping (more complex, cinema-grade)
    Aces,
    /// No tone-mapping (pass-through for native HDR content)
    None,
}

/// Tone-mapper for converting SDR content to HDR output.
pub struct ToneMapper {
    mode: ToneMapperMode,
    hdr_peak_nits: f32,
}

impl Default for ToneMapper {
    fn default() -> Self {
        Self {
            mode: ToneMapperMode::Reinhard,
            hdr_peak_nits: 1000.0,
        }
    }
}

impl ToneMapper {
    pub fn new(mode: ToneMapperMode, hdr_peak_nits: f32) -> Self {
        Self {
            mode,
            hdr_peak_nits,
        }
    }

    /// Tone-map an SDR color (sRGB, 0..1) to HDR (rec2020/scRGB, 0..peak_nits).
    /// This is a placeholder; real tone-mapping requires more sophisticated algorithms.
    pub fn tone_map(&self, sdr_value: f32) -> f32 {
        match self.mode {
            ToneMapperMode::Reinhard => {
                // Reinhard: map [0,1] → [0,peak_nits] with smooth knee
                let peak = self.hdr_peak_nits / 80.0; // Normalize to ~80 nits baseline
                (sdr_value * peak) / (1.0 + sdr_value * peak)
            }
            ToneMapperMode::Aces => {
                // Placeholder ACES; real ACES is more complex
                let peak = self.hdr_peak_nits / 80.0;
                (sdr_value * (peak + 0.5)) / (1.0 + sdr_value)
            }
            ToneMapperMode::None => sdr_value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_space_serialize() {
        assert_eq!(ColorSpace::SRgb.as_str(), "srgb");
        assert_eq!(ColorSpace::Rec2020.as_str(), "rec2020");
        assert_eq!(ColorSpace::ScRgb.as_str(), "scrgb");
    }

    #[test]
    fn test_color_space_deserialize() {
        assert_eq!(ColorSpace::from_str("srgb"), Some(ColorSpace::SRgb));
        assert_eq!(ColorSpace::from_str("rec2020"), Some(ColorSpace::Rec2020));
        assert_eq!(ColorSpace::from_str("scrgb"), Some(ColorSpace::ScRgb));
        assert_eq!(ColorSpace::from_str("invalid"), None);
    }

    #[test]
    fn test_hdr_capabilities_default() {
        let caps = HdrCapabilities::default();
        assert!(!caps.hdr_supported);
        assert_eq!(caps.current_color_space, ColorSpace::SRgb);
    }

    #[test]
    fn test_tone_mapper_reinhard() {
        let mapper = ToneMapper::new(ToneMapperMode::Reinhard, 1000.0);
        // Reinhard should map 0.5 (mid gray) to a reasonable HDR value
        let result = mapper.tone_map(0.5);
        assert!(result > 0.0 && result < 1.0);
    }

    #[test]
    fn test_tone_mapper_none() {
        let mapper = ToneMapper::new(ToneMapperMode::None, 1000.0);
        // None mode should pass through unchanged
        assert_eq!(mapper.tone_map(0.5), 0.5);
    }
}
