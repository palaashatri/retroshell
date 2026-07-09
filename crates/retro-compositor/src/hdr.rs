//! HDR and color space management for the compositor.

use std::collections::HashMap;

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
        match s {
            "srgb" => Some(Self::SRgb),
            "rec2020" => Some(Self::Rec2020),
            "scrgb" => Some(Self::ScRgb),
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
    /// Detect HDR capabilities from the GPU.
    /// Returns capabilities detected; falls back to SDR if HDR unavailable.
    pub fn detect() -> Self {
        // In a real implementation, query wgpu adapter for:
        // - RGBA16F support (Rec2020, scRGB)
        // - HDR texture formats
        // - Compositor color space negotiation capability
        //
        // For now: default to SDR (sRGB) with infrastructure for future GPU detection.
        let hdr_supported = false; // Placeholder; detect GPU in real implementation
        let supported_color_spaces = if hdr_supported {
            vec![ColorSpace::SRgb, ColorSpace::Rec2020, ColorSpace::ScRgb]
        } else {
            vec![ColorSpace::SRgb]
        };

        Self {
            hdr_supported,
            supported_color_spaces,
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
