use image::codecs::png::PngEncoder;
use image::{ImageEncoder, RgbaImage};
use snapdown_core::domain::finding::Region;
use xcap::Monitor;

use crate::error::CaptureError;

pub struct RegionCapturer;

impl RegionCapturer {
    pub fn capture_monitor_image(
        source_monitor: Option<&str>,
    ) -> Result<(RgbaImage, u32, u32), CaptureError> {
        let monitors = Monitor::all().map_err(|e| {
            let msg = e.to_string();
            if msg.contains("No display") || msg.contains("empty") {
                CaptureError::NoDisplayFound
            } else {
                CaptureError::CaptureFailed(msg)
            }
        })?;

        if monitors.is_empty() {
            return Err(CaptureError::NoDisplayFound);
        }

        let target_monitor = if let Some(target_name) = source_monitor {
            if target_name.is_empty() {
                monitors
                    .iter()
                    .find(|m| m.is_primary().unwrap_or(false))
                    .or_else(|| monitors.first())
                    .ok_or(CaptureError::NoDisplayFound)?
            } else {
                monitors
                    .iter()
                    .find(|m| {
                        if let Ok(name) = m.name() {
                            name.eq_ignore_ascii_case(target_name)
                        } else {
                            false
                        }
                    })
                    .or_else(|| {
                        monitors.iter().find(|m| {
                            if let Ok(name) = m.name() {
                                name.contains(target_name) || target_name.contains(&name)
                            } else {
                                false
                            }
                        })
                    })
                    .or_else(|| {
                        if target_name.eq_ignore_ascii_case("DISPLAY1")
                            || target_name.starts_with("DISPLAY")
                        {
                            monitors
                                .iter()
                                .find(|m| m.is_primary().unwrap_or(false))
                                .or_else(|| monitors.first())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| CaptureError::MonitorNotFound(target_name.to_string()))?
            }
        } else {
            monitors
                .iter()
                .find(|m| m.is_primary().unwrap_or(false))
                .or_else(|| monitors.first())
                .ok_or(CaptureError::NoDisplayFound)?
        };

        let mon_w = target_monitor
            .width()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        let mon_h = target_monitor
            .height()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        let full_image = target_monitor
            .capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        Ok((full_image, mon_w, mon_h))
    }

    pub fn capture_region(
        region: &Region,
        source_monitor: Option<&str>,
    ) -> Result<Vec<u8>, CaptureError> {
        if region.width < 8 || region.height < 8 {
            return Err(CaptureError::InvalidRegion(
                "Region must be at least 8x8 pixels".to_string(),
            ));
        }

        let monitors = Monitor::all().map_err(|e| {
            let msg = e.to_string();
            if msg.contains("No display") || msg.contains("empty") {
                CaptureError::NoDisplayFound
            } else {
                CaptureError::CaptureFailed(msg)
            }
        })?;

        if monitors.is_empty() {
            return Err(CaptureError::NoDisplayFound);
        }

        let target_monitor = if let Some(target_name) = source_monitor {
            if target_name.is_empty() {
                monitors
                    .iter()
                    .find(|m| m.is_primary().unwrap_or(false))
                    .or_else(|| monitors.first())
                    .ok_or(CaptureError::NoDisplayFound)?
            } else {
                monitors
                    .iter()
                    .find(|m| {
                        if let Ok(name) = m.name() {
                            name.eq_ignore_ascii_case(target_name)
                        } else {
                            false
                        }
                    })
                    .or_else(|| {
                        monitors.iter().find(|m| {
                            if let Ok(name) = m.name() {
                                name.contains(target_name) || target_name.contains(&name)
                            } else {
                                false
                            }
                        })
                    })
                    .or_else(|| {
                        if target_name.eq_ignore_ascii_case("DISPLAY1")
                            || target_name.starts_with("DISPLAY")
                        {
                            monitors
                                .iter()
                                .find(|m| m.is_primary().unwrap_or(false))
                                .or_else(|| monitors.first())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| CaptureError::MonitorNotFound(target_name.to_string()))?
            }
        } else {
            monitors
                .iter()
                .find(|m| m.is_primary().unwrap_or(false))
                .or_else(|| monitors.first())
                .ok_or(CaptureError::NoDisplayFound)?
        };

        let mon_w = target_monitor
            .width()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        let mon_h = target_monitor
            .height()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        if region.x < 0
            || region.y < 0
            || (region.x as u64 + region.width as u64) > mon_w as u64
            || (region.y as u64 + region.height as u64) > mon_h as u64
        {
            return Err(CaptureError::RegionExceedsMonitorBounds {
                requested: format!(
                    "{},{},{},{}",
                    region.x, region.y, region.width, region.height
                ),
                monitor: format!("{mon_w}x{mon_h}"),
            });
        }

        let full_image = target_monitor
            .capture_image()
            .map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;

        Self::crop_and_encode_image(&full_image, region)
    }

    pub fn crop_and_encode_image(
        source: &RgbaImage,
        region: &Region,
    ) -> Result<Vec<u8>, CaptureError> {
        if region.width < 8 || region.height < 8 {
            return Err(CaptureError::InvalidRegion(
                "Region must be at least 8x8 pixels".to_string(),
            ));
        }

        let (src_w, src_h) = source.dimensions();

        if region.x < 0
            || region.y < 0
            || (region.x as u64 + region.width as u64) > src_w as u64
            || (region.y as u64 + region.height as u64) > src_h as u64
        {
            return Err(CaptureError::RegionExceedsMonitorBounds {
                requested: format!(
                    "{},{},{},{}",
                    region.x, region.y, region.width, region.height
                ),
                monitor: format!("{src_w}x{src_h}"),
            });
        }

        let cropped = image::imageops::crop_imm(
            source,
            region.x as u32,
            region.y as u32,
            region.width,
            region.height,
        )
        .to_image();

        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        encoder
            .write_image(
                cropped.as_raw(),
                cropped.width(),
                cropped.height(),
                image::ExtendedColorType::Rgba8,
            )
            .map_err(|e| CaptureError::EncodingFailed(e.to_string()))?;

        Ok(bytes)
    }
}
