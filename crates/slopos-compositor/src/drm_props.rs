//! Generic DRM/KMS property access, and the HDR / VRR properties built on it.
//!
//! Everything here talks to real kernel properties through `drm::control::Device`.
//! There is no simulation: if the connector does not expose `HDR_OUTPUT_METADATA`
//! or `vrr_capable`, the functions say so rather than pretending.
//!
//! References:
//! - `include/uapi/drm/drm_mode.h` — `struct hdr_output_metadata`,
//!   `struct hdr_metadata_infoframe`
//! - `drivers/gpu/drm/drm_connector.c` — the `Colorspace`, `max bpc`,
//!   `HDR_OUTPUT_METADATA` and `vrr_capable` property definitions
//! - CTA-861-G §6.9 — HDR Static Metadata Data Block / infoframe encoding
//!
//! Verifiability: a VirtualBox `vmwgfx` connector exposes none of these
//! properties, so in the VM every probe correctly reports "unsupported". Real
//! HDR/VRR behaviour can only be confirmed on hardware with a capable
//! connector (`sudo modetest -c` will list the properties).

#![cfg(target_os = "linux")]

use std::collections::HashMap;

// `drm` is not a direct dependency; use smithay's reexport so the version
// always matches the one smithay's DrmDevice was built against.
use smithay::reexports::drm;

use drm::control::{property, Device as ControlDevice, ResourceHandle};

/// Kernel EOTF values for `hdr_metadata_infoframe.eotf` (CTA-861-G Table 85).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Eotf {
    /// Traditional gamma, SDR luminance range.
    TraditionalSdr = 0,
    /// Traditional gamma, HDR luminance range.
    TraditionalHdr = 1,
    /// SMPTE ST 2084 (PQ) — what HDR10 uses.
    St2084 = 2,
    /// ITU-R BT.2100 Hybrid Log-Gamma.
    Hlg = 3,
}

/// `struct hdr_metadata_infoframe` from `include/uapi/drm/drm_mode.h`.
///
/// Units, straight from the CTA-861 encoding:
/// - `display_primaries` / `white_point`: CIE 1931 xy in 0.00002 steps
///   (i.e. `raw = xy * 50000`)
/// - `max_display_mastering_luminance`: nits, 1-nit steps
/// - `min_display_mastering_luminance`: nits in 0.0001 steps (`raw = nits * 10000`)
/// - `max_cll` / `max_fall`: nits, 1-nit steps
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HdrMetadataInfoframe {
    pub eotf: u8,
    /// Always 0 = Static Metadata Type 1.
    pub metadata_type: u8,
    /// R, G, B primaries as (x, y) pairs, in that order.
    pub display_primaries: [ChromaticityPoint; 3],
    pub white_point: ChromaticityPoint,
    pub max_display_mastering_luminance: u16,
    pub min_display_mastering_luminance: u16,
    pub max_cll: u16,
    pub max_fall: u16,
}

/// One CIE 1931 xy coordinate pair in 0.00002 units.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ChromaticityPoint {
    pub x: u16,
    pub y: u16,
}

impl ChromaticityPoint {
    /// Encode CIE xy floats (0.0..=1.0) into the kernel's 0.00002 units.
    pub fn from_xy(x: f32, y: f32) -> Self {
        Self {
            x: (x.clamp(0.0, 1.0) * 50_000.0).round() as u16,
            y: (y.clamp(0.0, 1.0) * 50_000.0).round() as u16,
        }
    }
}

/// `struct hdr_output_metadata` — the payload of the `HDR_OUTPUT_METADATA` blob.
///
/// The kernel's definition is a `__u32` tag followed by a union whose only
/// current member is `hdmi_metadata_type1`.
#[repr(C)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct HdrOutputMetadata {
    /// 0 = HDMI_STATIC_METADATA_TYPE1.
    pub metadata_type: u32,
    pub hdmi_metadata_type1: HdrMetadataInfoframe,
}

impl HdrOutputMetadata {
    /// HDR10: ST 2084 (PQ) transfer, BT.2020 primaries, D65 white point.
    ///
    /// `max_mastering_nits` / `min_mastering_nits` describe the mastering
    /// display; `max_cll` (content light level) and `max_fall` (frame-average
    /// light level) describe the content. Values of 0 mean "unknown", which is
    /// legal and tells the sink to use its own defaults.
    pub fn hdr10(
        max_mastering_nits: u16,
        min_mastering_nits: f32,
        max_cll: u16,
        max_fall: u16,
    ) -> Self {
        Self {
            metadata_type: 0,
            hdmi_metadata_type1: HdrMetadataInfoframe {
                eotf: Eotf::St2084 as u8,
                metadata_type: 0,
                // BT.2020 primaries.
                display_primaries: [
                    ChromaticityPoint::from_xy(0.708, 0.292), // R
                    ChromaticityPoint::from_xy(0.170, 0.797), // G
                    ChromaticityPoint::from_xy(0.131, 0.046), // B
                ],
                // D65.
                white_point: ChromaticityPoint::from_xy(0.3127, 0.3290),
                max_display_mastering_luminance: max_mastering_nits,
                min_display_mastering_luminance: (min_mastering_nits * 10_000.0).round() as u16,
                max_cll,
                max_fall,
            },
        }
    }

    /// SDR: traditional gamma. Setting this (rather than clearing the property)
    /// is how a compositor takes a display back out of HDR mode.
    pub fn sdr() -> Self {
        Self {
            metadata_type: 0,
            hdmi_metadata_type1: HdrMetadataInfoframe {
                eotf: Eotf::TraditionalSdr as u8,
                metadata_type: 0,
                ..Default::default()
            },
        }
    }
}

/// One property as the kernel reports it for a specific object.
#[derive(Debug, Clone)]
pub struct PropEntry {
    pub handle: property::Handle,
    pub name: String,
    pub value_type: property::ValueType,
    /// Current raw value on the object this index was built from.
    pub raw_value: u64,
}

impl PropEntry {
    /// For an enum property, the symbolic name of the current value.
    pub fn enum_name(&self) -> Option<String> {
        match &self.value_type {
            property::ValueType::Enum(values) => values
                .get_value_from_raw_value(self.raw_value)
                .map(|v| v.name().to_string_lossy().into_owned()),
            _ => None,
        }
    }

    /// For an enum property, the raw value matching a symbolic name.
    pub fn enum_value(&self, name: &str) -> Option<u64> {
        match &self.value_type {
            property::ValueType::Enum(values) => {
                let (_, enums) = values.values();
                enums
                    .iter()
                    .find(|e| e.name().to_string_lossy() == name)
                    .map(|e| e.value())
            }
            _ => None,
        }
    }

    /// For a range property, its (min, max).
    pub fn range(&self) -> Option<(u64, u64)> {
        match self.value_type {
            property::ValueType::UnsignedRange(lo, hi) => Some((lo, hi)),
            _ => None,
        }
    }
}

/// All properties of one DRM object, indexed by name.
#[derive(Debug, Clone, Default)]
pub struct PropertyIndex {
    by_name: HashMap<String, PropEntry>,
}

impl PropertyIndex {
    /// Read every property of `handle` from `device`.
    pub fn read<D, H>(device: &D, handle: H) -> std::io::Result<Self>
    where
        D: ControlDevice,
        H: ResourceHandle,
    {
        let set = device.get_properties(handle)?;
        let (handles, raw_values) = set.as_props_and_values();
        let mut by_name = HashMap::with_capacity(handles.len());
        for (h, raw) in handles.iter().zip(raw_values.iter()) {
            let info = device.get_property(*h)?;
            let name = info.name().to_string_lossy().into_owned();
            by_name.insert(
                name.clone(),
                PropEntry {
                    handle: *h,
                    name,
                    value_type: info.value_type(),
                    raw_value: *raw,
                },
            );
        }
        Ok(Self { by_name })
    }

    pub fn get(&self, name: &str) -> Option<&PropEntry> {
        self.by_name.get(name)
    }

    pub fn has(&self, name: &str) -> bool {
        self.by_name.contains_key(name)
    }

    /// Property names, sorted — useful for logging what a connector supports.
    pub fn names(&self) -> Vec<&str> {
        let mut v: Vec<&str> = self.by_name.keys().map(String::as_str).collect();
        v.sort_unstable();
        v
    }
}

/// Set a property by name on a DRM object.
///
/// Returns `Ok(false)` when the object does not expose that property, which is
/// the normal case on hardware (and in a VM) that cannot do HDR or VRR.
pub fn set_property_by_name<D, H>(
    device: &D,
    handle: H,
    index: &PropertyIndex,
    name: &str,
    value: u64,
) -> std::io::Result<bool>
where
    D: ControlDevice,
    H: ResourceHandle,
{
    let Some(entry) = index.get(name) else {
        return Ok(false);
    };
    device.set_property(handle, entry.handle, value)?;
    Ok(true)
}

// ---------------------------------------------------------------------------
// VRR
// ---------------------------------------------------------------------------

/// What the kernel says about variable refresh on one connector/CRTC pair.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VrrState {
    /// Connector exposes `vrr_capable` and it reads non-zero.
    pub capable: bool,
    /// CRTC exposes `VRR_ENABLED`.
    pub controllable: bool,
    /// Current value of `VRR_ENABLED`.
    pub enabled: bool,
}

/// Read `vrr_capable` (connector) and `VRR_ENABLED` (CRTC).
///
/// Property names are exactly as the kernel spells them — lowercase on the
/// connector, uppercase on the CRTC. That asymmetry is real, not a typo.
pub fn probe_vrr(connector_props: &PropertyIndex, crtc_props: &PropertyIndex) -> VrrState {
    let capable = connector_props
        .get("vrr_capable")
        .map(|p| p.raw_value != 0)
        .unwrap_or(false);
    let enabled_prop = crtc_props.get("VRR_ENABLED");
    VrrState {
        capable,
        controllable: enabled_prop.is_some(),
        enabled: enabled_prop.map(|p| p.raw_value != 0).unwrap_or(false),
    }
}

/// Turn variable refresh on or off for a CRTC.
///
/// Refuses when the connector is not `vrr_capable`, so we never claim VRR on a
/// display that cannot do it.
pub fn set_vrr_enabled<D>(
    device: &D,
    crtc: drm::control::crtc::Handle,
    crtc_props: &PropertyIndex,
    state: VrrState,
    enable: bool,
) -> std::io::Result<bool>
where
    D: ControlDevice,
{
    if enable && !state.capable {
        return Ok(false);
    }
    set_property_by_name(
        device,
        crtc,
        crtc_props,
        "VRR_ENABLED",
        if enable { 1 } else { 0 },
    )
}

// ---------------------------------------------------------------------------
// HDR
// ---------------------------------------------------------------------------

/// What the kernel says about HDR on one connector.
#[derive(Debug, Clone, Default)]
pub struct HdrConnectorCaps {
    /// Connector exposes the `HDR_OUTPUT_METADATA` blob property.
    pub has_hdr_metadata: bool,
    /// Connector exposes `Colorspace` and it lists a BT.2020 RGB entry.
    pub has_bt2020_colorspace: bool,
    /// Highest value the `max bpc` property allows (HDR10 needs >= 10).
    pub max_bpc: Option<u64>,
    /// Every `Colorspace` value the connector advertises.
    pub colorspaces: Vec<String>,
}

impl HdrConnectorCaps {
    /// True when the connector can actually be driven in HDR10.
    pub fn hdr10_capable(&self) -> bool {
        self.has_hdr_metadata && self.has_bt2020_colorspace && self.max_bpc.unwrap_or(8) >= 10
    }

    /// One honest line for logs.
    pub fn summary(&self) -> String {
        format!(
            "hdr_metadata={} bt2020_colorspace={} max_bpc={} => hdr10_capable={}",
            self.has_hdr_metadata,
            self.has_bt2020_colorspace,
            self.max_bpc
                .map(|v| v.to_string())
                .unwrap_or_else(|| "n/a".into()),
            self.hdr10_capable()
        )
    }
}

/// The `Colorspace` enum entry we want for HDR10 output.
pub const COLORSPACE_BT2020_RGB: &str = "BT2020_RGB";
/// The `Colorspace` entry to return to for SDR.
pub const COLORSPACE_DEFAULT: &str = "Default";

/// Inspect a connector's HDR-related properties.
pub fn probe_hdr(connector_props: &PropertyIndex) -> HdrConnectorCaps {
    let colorspace = connector_props.get("Colorspace");
    let colorspaces: Vec<String> = colorspace
        .and_then(|p| match &p.value_type {
            property::ValueType::Enum(values) => {
                let (_, enums) = values.values();
                Some(
                    enums
                        .iter()
                        .map(|e| e.name().to_string_lossy().into_owned())
                        .collect(),
                )
            }
            _ => None,
        })
        .unwrap_or_default();

    HdrConnectorCaps {
        has_hdr_metadata: connector_props.has("HDR_OUTPUT_METADATA"),
        has_bt2020_colorspace: colorspaces.iter().any(|c| c == COLORSPACE_BT2020_RGB),
        max_bpc: connector_props
            .get("max bpc")
            .and_then(|p| p.range().map(|(_, hi)| hi)),
        colorspaces,
    }
}

/// Drive a connector into HDR10: publish the mastering metadata blob, switch
/// `Colorspace` to BT.2020 RGB, and raise `max bpc` to 10.
///
/// 8 bits per channel cannot carry a PQ signal without visible banding, which
/// is why `max bpc` must be raised as part of the same change.
///
/// Returns the blob handle, which the caller must keep and later destroy with
/// [`clear_hdr`] — the kernel holds a reference until the property stops using it.
pub fn apply_hdr10<D>(
    device: &D,
    connector: drm::control::connector::Handle,
    props: &PropertyIndex,
    metadata: &HdrOutputMetadata,
) -> std::io::Result<Option<property::Value<'static>>>
where
    D: ControlDevice,
{
    let caps = probe_hdr(props);
    if !caps.hdr10_capable() {
        return Ok(None);
    }

    if let Some(entry) = props.get("max bpc") {
        if let Some((_, hi)) = entry.range() {
            let target = hi.min(10);
            device.set_property(connector, entry.handle, target)?;
        }
    }

    if let Some(entry) = props.get("Colorspace") {
        if let Some(raw) = entry.enum_value(COLORSPACE_BT2020_RGB) {
            device.set_property(connector, entry.handle, raw)?;
        }
    }

    let blob = device.create_property_blob(metadata)?;
    if let Some(entry) = props.get("HDR_OUTPUT_METADATA") {
        device.set_property(connector, entry.handle, blob.into())?;
    }
    Ok(Some(blob))
}

/// Return a connector to SDR: traditional-gamma metadata and the default
/// colorspace, then release the previous blob.
pub fn clear_hdr<D>(
    device: &D,
    connector: drm::control::connector::Handle,
    props: &PropertyIndex,
    previous_blob: Option<property::Value<'static>>,
) -> std::io::Result<()>
where
    D: ControlDevice,
{
    if let Some(entry) = props.get("Colorspace") {
        if let Some(raw) = entry.enum_value(COLORSPACE_DEFAULT) {
            device.set_property(connector, entry.handle, raw)?;
        }
    }
    let sdr = HdrOutputMetadata::sdr();
    let blob = device.create_property_blob(&sdr)?;
    if let Some(entry) = props.get("HDR_OUTPUT_METADATA") {
        device.set_property(connector, entry.handle, blob.into())?;
    }
    if let Some(old) = previous_blob {
        let _ = device.destroy_property_blob(old.into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hdr_output_metadata_matches_kernel_layout() {
        // struct hdr_metadata_infoframe: 2 u8 + 3*(2 u16) + 2 u16 + 4 u16 = 26 bytes.
        assert_eq!(core::mem::size_of::<HdrMetadataInfoframe>(), 26);
        // struct hdr_output_metadata: u32 tag + the infoframe, aligned to 4.
        assert_eq!(core::mem::align_of::<HdrOutputMetadata>(), 4);
        assert_eq!(core::mem::size_of::<HdrOutputMetadata>(), 32);
    }

    #[test]
    fn chromaticity_encodes_in_0_00002_units() {
        // BT.2020 red primary x = 0.708 -> 0.708 * 50000 = 35400
        let p = ChromaticityPoint::from_xy(0.708, 0.292);
        assert_eq!(p.x, 35_400);
        assert_eq!(p.y, 14_600);
        // D65 white point.
        let w = ChromaticityPoint::from_xy(0.3127, 0.3290);
        assert_eq!(w.x, 15_635);
        assert_eq!(w.y, 16_450);
    }

    #[test]
    fn hdr10_uses_pq_and_bt2020_primaries() {
        let md = HdrOutputMetadata::hdr10(1000, 0.005, 1000, 400);
        assert_eq!(md.metadata_type, 0);
        assert_eq!(md.hdmi_metadata_type1.eotf, Eotf::St2084 as u8);
        assert_eq!(md.hdmi_metadata_type1.metadata_type, 0);
        assert_eq!(md.hdmi_metadata_type1.max_display_mastering_luminance, 1000);
        // min luminance is in 0.0001-nit units: 0.005 nits -> 50
        assert_eq!(md.hdmi_metadata_type1.min_display_mastering_luminance, 50);
        assert_eq!(md.hdmi_metadata_type1.max_cll, 1000);
        assert_eq!(md.hdmi_metadata_type1.max_fall, 400);
        assert_eq!(
            md.hdmi_metadata_type1.display_primaries[0],
            ChromaticityPoint::from_xy(0.708, 0.292)
        );
    }

    #[test]
    fn sdr_metadata_uses_traditional_gamma() {
        let md = HdrOutputMetadata::sdr();
        assert_eq!(md.hdmi_metadata_type1.eotf, Eotf::TraditionalSdr as u8);
    }

    #[test]
    fn hdr10_capability_requires_all_three_conditions() {
        let full = HdrConnectorCaps {
            has_hdr_metadata: true,
            has_bt2020_colorspace: true,
            max_bpc: Some(12),
            colorspaces: vec![COLORSPACE_BT2020_RGB.into()],
        };
        assert!(full.hdr10_capable());

        let eight_bpc = HdrConnectorCaps {
            max_bpc: Some(8),
            ..full.clone()
        };
        assert!(!eight_bpc.hdr10_capable(), "8 bpc cannot carry PQ");

        let no_blob = HdrConnectorCaps {
            has_hdr_metadata: false,
            ..full.clone()
        };
        assert!(!no_blob.hdr10_capable());

        let no_colorspace = HdrConnectorCaps {
            has_bt2020_colorspace: false,
            ..full
        };
        assert!(!no_colorspace.hdr10_capable());
    }

    #[test]
    fn vrr_probe_reports_unsupported_when_properties_absent() {
        // A vmwgfx connector in a VM: neither property exists.
        let empty = PropertyIndex::default();
        let state = probe_vrr(&empty, &empty);
        assert!(!state.capable);
        assert!(!state.controllable);
        assert!(!state.enabled);
    }

    #[test]
    fn hdr_probe_reports_unsupported_when_properties_absent() {
        let empty = PropertyIndex::default();
        let caps = probe_hdr(&empty);
        assert!(!caps.hdr10_capable());
        assert!(caps.colorspaces.is_empty());
        assert!(caps.summary().contains("hdr10_capable=false"));
    }
}
