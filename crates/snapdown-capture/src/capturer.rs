use image::codecs::png::PngEncoder;
use image::{ImageEncoder, RgbaImage};
use snapdown_core::domain::finding::Region;
use xcap::Monitor;

use crate::error::CaptureError;

/// One monitor's own pixels, plus where that monitor sits in virtual-desktop coordinates.
///
/// The grab is per-monitor because the OS is: there is no single call that hands back the whole
/// virtual desktop. The overlay that *presents* them is a single window over the whole desktop -
/// one window per monitor was tried and reverted, because each carried its own renderer warm-up
/// and its own event-loop turn (`BUG-26`, `BUG-27`). Do not read this type as an argument for
/// going back.
pub struct MonitorCapture {
    pub image: RgbaImage,
    pub width: u32,
    pub height: u32,
    /// Top-left of this monitor in virtual-desktop coordinates. Negative on monitors placed
    /// left of, or above, the primary one.
    pub origin_x: i32,
    pub origin_y: i32,
    /// This monitor's own DPI scale factor (1.5 at 150%, 1.75 at 175%). `width`/`height` above
    /// are *physical* pixels; a toolkit that sizes windows in logical pixels needs this to
    /// convert, and it cannot be asked of the window itself before that window exists.
    pub scale_factor: f32,
    /// The OS display name, recorded on the Finding so a capture can be traced to its screen.
    pub name: String,
}

/// Where one monitor sits inside a [`VirtualDesktopCapture`], in that capture's own physical
/// pixel space (its top-left is the virtual desktop's top-left, so these are never negative).
///
/// The overlay uses these to confine the crosshair to the monitor under the pointer and to clamp
/// a selection to that monitor, which is what keeps a single window from producing a region that
/// spans two screens.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    /// The OS display name, recorded on the Finding so a capture can be traced to its screen.
    pub name: String,
}

/// One rectangle a single click may take, and how near the front of the z-order it sits.
///
/// The depth is what makes occlusion work. Candidates are gathered per window, by handle, so a
/// window buried behind another still reports its rectangles - and offering those would highlight
/// something the Reviewer cannot even see. Keeping the enumeration order (`EnumWindows` walks the
/// z-order from front to back) lets a hit test prefer whatever is actually on top at that point.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureTarget {
    pub region: Region,
    /// 0 is the frontmost window; larger is further back.
    pub depth: u32,
}

/// The whole virtual desktop: every monitor's pixels, plus the canvas geometry they add up to.
///
/// There is deliberately **no stitched canvas here**. This used to own one, and building it cost
/// the Reviewer most of the wait between pressing Capture and the screen freezing: a canvas
/// allocation, an `image::imageops::overlay` per monitor - a per-pixel *blend* loop, not a blit -
/// and then a full copy into the toolkit's own pixel buffer. [`Self::blit_into`] writes the
/// monitors straight into the buffer that will be presented instead, which removed the blend and
/// the copy together: measured in release on a 6000x3840 two-monitor desktop, 83-91 ms became
/// 36-38 ms. See `BUG-28`.
///
/// The pixels stay at native physical resolution either way, so crops are still full quality.
pub struct VirtualDesktopCapture {
    /// Each monitor's own grab, unstitched, in the order [`RegionCapturer::capture_each_monitor`]
    /// returns them.
    pub captures: Vec<MonitorCapture>,
    pub width: u32,
    pub height: u32,
    /// Top-left of the virtual desktop in virtual-desktop coordinates. Negative when a monitor
    /// sits left of, or above, the primary one.
    pub origin_x: i32,
    pub origin_y: i32,
    pub monitors: Vec<MonitorRect>,
}

/// RGBA8: four bytes per pixel, in both the monitor grabs and the canvas they are blitted into.
const BYTES_PER_PIXEL: usize = 4;

impl VirtualDesktopCapture {
    /// How many bytes a buffer must hold to take the whole canvas.
    pub fn byte_len(&self) -> usize {
        self.width as usize * self.height as usize * BYTES_PER_PIXEL
    }

    /// Writes every monitor's pixels into `dst` - the canvas, RGBA8, row-major, `width` pixels per
    /// row - at that monitor's place in it.
    ///
    /// `dst` is meant to be the buffer that will actually be shown, which is the whole point: the
    /// desktop's pixels are not blended into a canvas and then copied out of it. The blend was
    /// arithmetic whose answer was already the source pixel - every source pixel is opaque and the
    /// destination untouched - and `identical_bytes_to_the_stitch_it_replaces` is the test that
    /// keeps that equivalence honest rather than merely claimed.
    ///
    /// Measured in release on a 6000x3840 two-monitor desktop, all shapes producing identical
    /// bytes: the old canvas-plus-blend-plus-copy took 83-91 ms, and this takes 36-38 ms with a
    /// freshly allocated `dst`.
    ///
    /// **Most of that 36-38 ms is not this function.** Allocating a zeroed 92 MB
    /// `SharedPixelBuffer` costs 33-37 ms on its own, because `SharedVector`'s `FromIterator` is a
    /// per-element push loop rather than a `calloc` - a plain 92 MB write pass is 3.6 ms by
    /// comparison. Handing this the SAME buffer again on a later capture, while nothing else holds
    /// a clone of it, costs **4.3-4.6 ms** total. That is the cheap win still on the table, and it
    /// belongs to the caller, not here.
    ///
    /// Pixels no monitor covers are left exactly as the caller set them. A zeroed buffer therefore
    /// keeps the black gap that a non-rectangular desktop has always shown, and this function needs
    /// no notion of a gap at all.
    ///
    /// A monitor that does not fit the canvas is **skipped**, not panicked on. A display can be
    /// unplugged between the enumeration that sized the canvas and the grab, and an out-of-range
    /// index here would take the tray, the hotkeys and the Editor down with it - they share this
    /// one process (`AD-11`).
    pub fn blit_into(&self, dst: &mut [u8]) -> Result<(), CaptureError> {
        if dst.len() != self.byte_len() {
            return Err(CaptureError::InvalidRegion(format!(
                "canvas buffer holds {} bytes, but {}x{} needs {}",
                dst.len(),
                self.width,
                self.height,
                self.byte_len()
            )));
        }

        let canvas_width = self.width as usize;
        let canvas_height = self.height as usize;

        for capture in &self.captures {
            let (Ok(left), Ok(top)) = (
                usize::try_from(capture.origin_x - self.origin_x),
                usize::try_from(capture.origin_y - self.origin_y),
            ) else {
                continue;
            };

            let (width, height) = (capture.width as usize, capture.height as usize);
            if left + width > canvas_width || top + height > canvas_height {
                continue;
            }

            let row_bytes = width * BYTES_PER_PIXEL;
            let src = capture.image.as_raw();
            if src.len() < row_bytes * height {
                continue;
            }

            // A monitor as wide as the canvas occupies whole rows, so its destination is one
            // contiguous run - which is every single-monitor desktop. Worth about 1 ms per 33 MB
            // over the row loop below; small, and it costs exactly one branch.
            if left == 0 && width == canvas_width {
                let at = top * canvas_width * BYTES_PER_PIXEL;
                dst[at..at + row_bytes * height].copy_from_slice(&src[..row_bytes * height]);
                continue;
            }

            for row in 0..height {
                let to = ((top + row) * canvas_width + left) * BYTES_PER_PIXEL;
                let from = row * row_bytes;
                dst[to..to + row_bytes].copy_from_slice(&src[from..from + row_bytes]);
            }
        }

        Ok(())
    }
}

pub struct RegionCapturer;

impl RegionCapturer {
    /// Captures every attached monitor separately, each at its own native resolution.
    ///
    /// Prefer this over [`Self::capture_monitor_image`] for anything that has to *display* the
    /// capture: that one stitches its own canvas internally, which is the cost `BUG-28` is about.
    /// [`Self::capture_virtual_desktop`] wraps this with the desktop geometry and leaves the
    /// stitching to [`VirtualDesktopCapture::blit_into`].
    ///
    /// A monitor whose grab fails is skipped rather than failing the whole capture; the error is
    /// only returned when no monitor could be captured at all.
    pub fn capture_each_monitor() -> Result<Vec<MonitorCapture>, CaptureError> {
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

        // One thread per monitor. The grab dominates the time the user waits for the overlay to
        // appear, and it is a per-display operation with no shared state, so the displays are
        // captured concurrently rather than one after the other.
        let handles: Vec<_> = (0..monitors.len())
            .map(|index| std::thread::spawn(move || Self::capture_one_monitor(index)))
            .collect();

        let mut captures = Vec::with_capacity(handles.len());
        let mut last_error: Option<String> = None;

        for handle in handles {
            match handle.join() {
                Ok(Ok(capture)) => captures.push(capture),
                Ok(Err(e)) => last_error = Some(e),
                Err(_) => last_error = Some("a monitor capture thread panicked".to_string()),
            }
        }

        if captures.is_empty() {
            return Err(CaptureError::CaptureFailed(
                last_error.unwrap_or_else(|| "no monitor could be captured".to_string()),
            ));
        }

        // Deterministic order regardless of which thread finished first, so the overlay list and
        // the recorded monitor names do not shuffle between captures.
        captures.sort_by_key(|c| (c.origin_x, c.origin_y));

        Ok(captures)
    }

    /// The virtual desktop's bounds as `(origin_x, origin_y, width, height)` in physical pixels,
    /// without grabbing a single pixel.
    ///
    /// Exists so the capture overlay can be created and placed at application start, long before
    /// there is anything to show in it. Getting the window into existence early matters: its
    /// geometry is corrected one event-loop turn after creation, and doing that at start-up means
    /// the correction is not something the user watches happen when they press Capture.
    pub fn virtual_desktop_bounds() -> Result<(i32, i32, u32, u32), CaptureError> {
        let monitors = Monitor::all().map_err(|e| CaptureError::CaptureFailed(e.to_string()))?;
        if monitors.is_empty() {
            return Err(CaptureError::NoDisplayFound);
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;
        let mut max_x = i32::MIN;
        let mut max_y = i32::MIN;

        for m in &monitors {
            let x = m.x().unwrap_or(0);
            let y = m.y().unwrap_or(0);
            let w = m.width().unwrap_or(0) as i32;
            let h = m.height().unwrap_or(0) as i32;
            min_x = min_x.min(x);
            min_y = min_y.min(y);
            max_x = max_x.max(x + w);
            max_y = max_y.max(y + h);
        }

        Ok((
            min_x,
            min_y,
            (max_x - min_x).max(1) as u32,
            (max_y - min_y).max(1) as u32,
        ))
    }

    /// Captures the whole virtual desktop as ONE image at native physical resolution, together
    /// with where each monitor sits inside it.
    ///
    /// This exists so the capture overlay can be a single window. Per-monitor overlay windows
    /// solved the mixed-DPI problem but bought a class of multi-window defects with it - each
    /// window has its own renderer to warm up (a visible blink) and its own event-loop turn (so
    /// closing them was not simultaneous).
    ///
    /// One window does not have to cost image quality. A window's surface maps 1:1 onto desktop
    /// pixels for a per-monitor-DPI-aware process, so a canvas at native physical resolution,
    /// drawn full-bleed into a window sized in physical pixels, is pixel-exact on every monitor
    /// regardless of their differing scale factors. The earlier attempt looked zoomed because the
    /// window was sized wrongly (physical numbers stored as logical, then multiplied by the scale
    /// factor), not because one window cannot represent two DPIs.
    ///
    /// Cropping comes straight out of this canvas, so saved output is full quality too.
    pub fn capture_virtual_desktop() -> Result<VirtualDesktopCapture, CaptureError> {
        let captures = Self::capture_each_monitor()?;

        let min_x = captures.iter().map(|c| c.origin_x).min().unwrap_or(0);
        let min_y = captures.iter().map(|c| c.origin_y).min().unwrap_or(0);
        let max_x = captures
            .iter()
            .map(|c| c.origin_x + c.width as i32)
            .max()
            .unwrap_or(0);
        let max_y = captures
            .iter()
            .map(|c| c.origin_y + c.height as i32)
            .max()
            .unwrap_or(0);

        // Window-local: the overlay's top-left is the virtual desktop's top-left, so a monitor's
        // offset inside the canvas is its virtual-desktop origin minus that.
        let monitors = captures
            .iter()
            .map(|capture| MonitorRect {
                x: capture.origin_x - min_x,
                y: capture.origin_y - min_y,
                width: capture.width,
                height: capture.height,
                name: capture.name.clone(),
            })
            .collect();

        // No canvas is built here. The caller allocates the buffer it is going to present and
        // passes it to `blit_into`, so the desktop's 92 MB is written once instead of being
        // blended into a canvas and then copied out of it - see `VirtualDesktopCapture`.
        Ok(VirtualDesktopCapture {
            captures,
            width: (max_x - min_x).max(1) as u32,
            height: (max_y - min_y).max(1) as u32,
            origin_x: min_x,
            origin_y: min_y,
            monitors,
        })
    }

    /// Grabs one monitor, tagging it with its placement, scale factor and display name.
    ///
    /// Takes an index and re-enumerates rather than accepting a `Monitor`: a `Monitor` owns a raw
    /// `HMONITOR` and so is not `Send`, and re-enumerating (an `EnumDisplayMonitors` call) is far
    /// cheaper than the grab this runs in parallel with. A monitor that vanished between the
    /// caller's enumeration and this one simply reports an error and is skipped.
    fn capture_one_monitor(index: usize) -> Result<MonitorCapture, String> {
        let m = Monitor::all()
            .map_err(|e| e.to_string())?
            .into_iter()
            .nth(index)
            .ok_or_else(|| format!("monitor {index} disappeared before it could be captured"))?;
        let image = m.capture_image().map_err(|e| e.to_string())?;
        let (width, height) = (image.width(), image.height());
        Ok(MonitorCapture {
            image,
            width,
            height,
            origin_x: m.x().unwrap_or(0),
            origin_y: m.y().unwrap_or(0),
            scale_factor: m.scale_factor().ok().filter(|s| *s > 0.0).unwrap_or(1.0),
            name: m
                .name()
                .ok()
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| format!("DISPLAY{}", index + 1)),
        })
    }

    /// Returns the captured image, its width and height, and the top-left origin of the
    /// captured area in virtual-desktop coordinates (nonzero whenever the captured monitor, or
    /// the stitched multi-monitor canvas, does not start at the primary monitor's origin).
    pub fn capture_monitor_image(
        source_monitor: Option<&str>,
    ) -> Result<(RgbaImage, u32, u32, i32, i32), CaptureError> {
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

        // If "ALL" or multi-monitor requested, stitch all monitors into one virtual canvas
        if source_monitor == Some("ALL") || (source_monitor.is_none() && monitors.len() > 1) {
            let mut min_x = i32::MAX;
            let mut min_y = i32::MAX;
            let mut max_x = i32::MIN;
            let mut max_y = i32::MIN;

            for m in &monitors {
                let x = m.x().unwrap_or(0);
                let y = m.y().unwrap_or(0);
                let w = m.width().unwrap_or(1920) as i32;
                let h = m.height().unwrap_or(1080) as i32;

                min_x = min_x.min(x);
                min_y = min_y.min(y);
                max_x = max_x.max(x + w);
                max_y = max_y.max(y + h);
            }

            let total_w = (max_x - min_x).max(1920) as u32;
            let total_h = (max_y - min_y).max(1080) as u32;

            let mut virtual_canvas = RgbaImage::new(total_w, total_h);

            for m in &monitors {
                if let Ok(img) = m.capture_image() {
                    let mx = m.x().unwrap_or(0) - min_x;
                    let my = m.y().unwrap_or(0) - min_y;
                    image::imageops::overlay(&mut virtual_canvas, &img, mx as i64, my as i64);
                }
            }

            return Ok((virtual_canvas, total_w, total_h, min_x, min_y));
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

        let origin_x = target_monitor.x().unwrap_or(0);
        let origin_y = target_monitor.y().unwrap_or(0);

        Ok((full_image, mon_w, mon_h, origin_x, origin_y))
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

    /// Crops a region out of a canvas held as raw RGBA8 bytes, `source_width` pixels per row.
    ///
    /// The desktop app's canvas lives in the toolkit's own pixel buffer rather than an `RgbaImage`,
    /// because owning a second copy of it is what `BUG-28` was about - see
    /// [`VirtualDesktopCapture::blit_into`]. So the crop cannot go through
    /// `image::imageops::crop_imm`, and copies its rows out itself. The crop is a selection, not a
    /// desktop, so this is cheap.
    ///
    /// Unlike [`Self::crop_and_encode_image`] this imposes no minimum size: the caller has already
    /// clamped the region into the canvas, and a small selection is the Reviewer's business.
    pub fn crop_rgba_from_slice(
        source: &[u8],
        source_width: u32,
        source_height: u32,
        region: &Region,
    ) -> Result<RgbaImage, CaptureError> {
        let expected = source_width as usize * source_height as usize * BYTES_PER_PIXEL;
        if source.len() != expected {
            return Err(CaptureError::InvalidRegion(format!(
                "canvas holds {} bytes, but {source_width}x{source_height} needs {expected}",
                source.len()
            )));
        }

        if region.x < 0
            || region.y < 0
            || region.width == 0
            || region.height == 0
            || (region.x as u64 + region.width as u64) > source_width as u64
            || (region.y as u64 + region.height as u64) > source_height as u64
        {
            return Err(CaptureError::RegionExceedsMonitorBounds {
                requested: format!(
                    "{},{},{},{}",
                    region.x, region.y, region.width, region.height
                ),
                monitor: format!("{source_width}x{source_height}"),
            });
        }

        let row_bytes = region.width as usize * BYTES_PER_PIXEL;
        let mut cropped = Vec::with_capacity(row_bytes * region.height as usize);
        for row in 0..region.height as usize {
            let from = ((region.y as usize + row) * source_width as usize + region.x as usize)
                * BYTES_PER_PIXEL;
            cropped.extend_from_slice(&source[from..from + row_bytes]);
        }

        RgbaImage::from_raw(region.width, region.height, cropped).ok_or_else(|| {
            CaptureError::InvalidRegion(
                "the cropped rows did not fill the requested region".to_string(),
            )
        })
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

    /// Every rectangle the Reviewer might plausibly want to capture, in virtual-desktop
    /// coordinates, ordered so that a hit test can take the first match: frontmost window first,
    /// and within one window the tightest container first.
    ///
    /// This is precomputed rather than asked per pointer move, and that is forced by the overlay
    /// rather than chosen. Once the capture overlay is up it covers the whole desktop and is the
    /// topmost window, so `ElementFromPoint` returns *our* window and a live query degrades to
    /// enumerating top-level windows. Enumerating by HANDLE instead makes z-order irrelevant to
    /// whether this works - which is what lets it run late, after the overlay is already up.
    ///
    /// Three filters decide what is worth offering, and each was added because its absence showed:
    ///
    /// - **Cloaked windows are skipped.** `IsWindowVisible` returns true for a Store/UWP window that
    ///   DWM is not drawing at all, so without `DWMWA_CLOAKED` the Reviewer is offered rectangles
    ///   belonging to windows that are not on screen.
    /// - **Off-screen UIA elements are skipped.** A scrolled-out or collapsed pane reports a
    ///   perfectly good bounding rectangle.
    /// - **Windows belonging to this process are skipped**, so the pre-warmed overlay does not offer
    ///   the whole desktop as a container.
    #[cfg(windows)]
    pub fn detect_capture_targets() -> Vec<CaptureTarget> {
        use windows::core::BOOL;
        use windows::Win32::Foundation::{HWND, LPARAM, RECT};
        use windows::Win32::Graphics::Dwm::{
            DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS,
        };
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        };
        use windows::Win32::System::Threading::GetCurrentProcessId;
        use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetDesktopWindow, GetShellWindow, GetWindowLongW, GetWindowRect,
            GetWindowThreadProcessId, IsIconic, IsWindowVisible, GWL_EXSTYLE, WS_EX_TOOLWINDOW,
        };

        /// A target smaller than this is a button or an icon, not something worth one click.
        const MIN_W: i32 = 64;
        const MIN_H: i32 = 40;
        /// How far into a window's own tree to look. Two levels reaches the panes and documents
        /// people target; deeper starts offering individual list rows.
        const MAX_DEPTH: u32 = 2;

        struct Scan {
            pid: u32,
            shell: HWND,
            desktop: HWND,
            windows: Vec<HWND>,
        }

        /// `EnumWindows` walks top-level windows front to back, so the push order IS the z-order.
        unsafe extern "system" fn collect(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let scan = &mut *(lparam.0 as *mut Scan);

            if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                return BOOL(1);
            }
            if hwnd == scan.shell || hwnd == scan.desktop {
                return BOOL(1);
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
            if pid == scan.pid {
                return BOOL(1);
            }
            if (GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 & WS_EX_TOOLWINDOW.0) != 0 {
                return BOOL(1);
            }
            // A cloaked window passes IsWindowVisible while DWM draws nothing for it - the usual
            // case being a suspended Store app. Offering its rectangles points at nothing.
            let mut cloaked = 0u32;
            if DwmGetWindowAttribute(
                hwnd,
                DWMWA_CLOAKED,
                &mut cloaked as *mut _ as *mut _,
                std::mem::size_of::<u32>() as u32,
            )
            .is_ok()
                && cloaked != 0
            {
                return BOOL(1);
            }
            scan.windows.push(hwnd);
            BOOL(1)
        }

        let mut targets: Vec<CaptureTarget> = Vec::new();

        unsafe {
            let mut scan = Scan {
                pid: GetCurrentProcessId(),
                shell: GetShellWindow(),
                desktop: GetDesktopWindow(),
                windows: Vec::new(),
            };
            let _ = EnumWindows(Some(collect), LPARAM(&mut scan as *mut _ as isize));

            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let automation: Option<IUIAutomation> =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER).ok();

            for (depth, hwnd) in scan.windows.into_iter().enumerate() {
                let depth = depth as u32;
                let mut window_targets: Vec<Region> = Vec::new();

                // The DWM frame bounds are what the Reviewer sees; `GetWindowRect` includes the
                // invisible resize border DWM adds, which would offer a target several pixels
                // larger than the window looks.
                let mut rect = RECT::default();
                let framed = DwmGetWindowAttribute(
                    hwnd,
                    DWMWA_EXTENDED_FRAME_BOUNDS,
                    &mut rect as *mut _ as *mut _,
                    std::mem::size_of::<RECT>() as u32,
                )
                .is_ok()
                    && rect.right > rect.left
                    && rect.bottom > rect.top;
                if !framed && GetWindowRect(hwnd, &mut rect).is_err() {
                    continue;
                }
                push_target(&mut window_targets, &rect, MIN_W, MIN_H);
                if window_targets.is_empty() {
                    // Too small to be worth a click, so its children are not worth walking.
                    continue;
                }

                if let Some(automation) = automation.as_ref() {
                    if let (Ok(element), Ok(walker)) = (
                        automation.ElementFromHandle(hwnd),
                        automation.ControlViewWalker(),
                    ) {
                        // Breadth-first, so a shallow useful pane is found before a deep one and
                        // the depth cap bites where it is meant to.
                        let mut level = vec![element];
                        for _ in 0..MAX_DEPTH {
                            let mut next = Vec::new();
                            for parent in &level {
                                let mut child = walker.GetFirstChildElement(parent).ok();
                                while let Some(node) = child {
                                    // A scrolled-out or collapsed pane still reports a perfectly
                                    // good rectangle, and offering it points at nothing.
                                    let on_screen = !node
                                        .CurrentIsOffscreen()
                                        .map(|v| v.as_bool())
                                        .unwrap_or(false);
                                    if on_screen {
                                        if let Ok(bounds) = node.CurrentBoundingRectangle() {
                                            push_target(&mut window_targets, &bounds, MIN_W, MIN_H);
                                        }
                                    }
                                    child = walker.GetNextSiblingElement(&node).ok();
                                    if on_screen {
                                        next.push(node);
                                    }
                                    // A pathological tree would otherwise stall the capture.
                                    if next.len() > 256 {
                                        break;
                                    }
                                }
                            }
                            if next.is_empty() {
                                break;
                            }
                            level = next;
                        }
                    }
                }

                // Tightest first WITHIN this window, so a hit test descends into the innermost
                // container it can before falling back to the window itself.
                window_targets.sort_by_key(|r| (r.width as u64) * (r.height as u64));
                targets.extend(
                    window_targets
                        .into_iter()
                        .map(|region| CaptureTarget { region, depth }),
                );
            }

            CoUninitialize();
        }

        // Frontmost window first. A hit test takes the first rectangle containing the pointer, so
        // a window buried behind another is never offered where the front one covers it.
        targets.sort_by_key(|t| (t.depth, (t.region.width as u64) * (t.region.height as u64)));
        targets
    }

    #[cfg(not(windows))]
    pub fn detect_capture_targets() -> Vec<CaptureTarget> {
        Vec::new()
    }
}

/// Adds `rect` to `targets` unless it is too small to be worth a click, or close enough to one
/// already collected that offering both would be a coin toss for the Reviewer.
///
/// The tolerance is deliberate: a window and its own top-level client pane routinely differ by a
/// pixel or two of border, and presenting those as two separate targets makes a one-click selection
/// feel unpredictable.
#[cfg(windows)]
fn push_target(
    targets: &mut Vec<Region>,
    rect: &windows::Win32::Foundation::RECT,
    min_w: i32,
    min_h: i32,
) {
    let (w, h) = (rect.right - rect.left, rect.bottom - rect.top);
    if w < min_w || h < min_h {
        return;
    }
    let candidate = Region::new(rect.left, rect.top, w as u32, h as u32);
    const NEAR: i32 = 3;
    if targets.iter().any(|t| {
        (t.x - candidate.x).abs() <= NEAR
            && (t.y - candidate.y).abs() <= NEAR
            && (t.width as i32 - candidate.width as i32).abs() <= NEAR * 2
            && (t.height as i32 - candidate.height as i32).abs() <= NEAR * 2
    }) {
        return;
    }
    targets.push(candidate);
}
