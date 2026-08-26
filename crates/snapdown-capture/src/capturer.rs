use image::codecs::png::PngEncoder;
use image::{ImageEncoder, RgbaImage};
use snapdown_core::domain::finding::Region;
use xcap::Monitor;

use crate::error::CaptureError;

/// One monitor's own pixels, plus where that monitor sits in virtual-desktop coordinates.
///
/// The capture overlay uses one of these per monitor rather than a single stitched canvas: on
/// Windows a window has exactly one DPI, so a window spanning displays with different scale
/// factors cannot render any of them 1:1 - it gets bitmap-scaled and, when it crosses the
/// boundary, `WM_DPICHANGED` re-derives its physical size and inflates it. Keeping each overlay
/// wholly inside one monitor sidesteps both.
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

/// The whole virtual desktop as one image, plus the monitors it spans.
pub struct VirtualDesktopCapture {
    /// Stitched at native physical resolution - no downscaling, so crops are full quality.
    pub image: RgbaImage,
    pub width: u32,
    pub height: u32,
    /// Top-left of the virtual desktop in virtual-desktop coordinates. Negative when a monitor
    /// sits left of, or above, the primary one.
    pub origin_x: i32,
    pub origin_y: i32,
    pub monitors: Vec<MonitorRect>,
}

pub struct RegionCapturer;

impl RegionCapturer {
    /// Captures every attached monitor separately, each at its own native resolution.
    ///
    /// Prefer this over [`Self::capture_monitor_image`] for anything that has to *display* the
    /// capture: it never allocates a stitched virtual canvas (which on a 6000x3840 desktop is a
    /// 92 MB zeroed allocation plus a blit per monitor), and it keeps each monitor's pixels in
    /// their own image so a per-monitor overlay can show them 1:1. See [`MonitorCapture`] for
    /// why per-monitor matters on mixed-DPI setups.
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

        let total_w = (max_x - min_x).max(1) as u32;
        let total_h = (max_y - min_y).max(1) as u32;

        let mut canvas = RgbaImage::new(total_w, total_h);
        let mut monitors = Vec::with_capacity(captures.len());

        for capture in &captures {
            // Window-local: the overlay's top-left is the virtual desktop's top-left, so a
            // monitor's offset inside the canvas is its virtual-desktop origin minus that.
            let local_x = capture.origin_x - min_x;
            let local_y = capture.origin_y - min_y;
            image::imageops::overlay(&mut canvas, &capture.image, local_x as i64, local_y as i64);
            monitors.push(MonitorRect {
                x: local_x,
                y: local_y,
                width: capture.width,
                height: capture.height,
                name: capture.name.clone(),
            });
        }

        Ok(VirtualDesktopCapture {
            image: canvas,
            width: total_w,
            height: total_h,
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

    #[cfg(windows)]
    pub fn detect_element_at_point(screen_x: i32, screen_y: i32) -> Option<Region> {
        use windows::core::BOOL;
        use windows::Win32::Foundation::{HWND, LPARAM, POINT, RECT};
        use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
        use windows::Win32::System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        };
        use windows::Win32::System::Threading::GetCurrentProcessId;
        use windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation};
        use windows::Win32::UI::WindowsAndMessaging::{
            EnumWindows, GetDesktopWindow, GetShellWindow, GetWindowLongW, GetWindowRect,
            GetWindowThreadProcessId, IsIconic, IsWindowVisible, GWL_EXSTYLE, GWL_STYLE,
            WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
        };

        unsafe {
            let pt = POINT {
                x: screen_x,
                y: screen_y,
            };

            let current_pid = GetCurrentProcessId();

            // Try modern Windows UI Automation with Container-Level walking
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let uia_result: windows::core::Result<IUIAutomation> =
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER);

            if let Ok(automation) = uia_result {
                if let Ok(element) = automation.ElementFromPoint(pt) {
                    if let Ok(elem_pid) = element.CurrentProcessId() {
                        if (elem_pid as u32) != current_pid {
                            // Find meaningful panel / viewport / toolbar container by inspecting element and its immediate ancestors
                            let mut best_region: Option<Region> = None;
                            let mut current_elem = Some(element);

                            let mut depth = 0;
                            while let Some(elem) = current_elem {
                                if depth > 6 {
                                    break;
                                }
                                depth += 1;

                                if let Ok(rect) = elem.CurrentBoundingRectangle() {
                                    let w = (rect.right - rect.left).max(0) as u32;
                                    let h = (rect.bottom - rect.top).max(0) as u32;

                                    // Check if this container represents a meaningful UI section (e.g. web body, toolbar, split pane, card)
                                    // Minimum size 64x40 to avoid tiny icon buttons/links, maximum under full screen canvas (w < 3800 || h < 2100)
                                    if w >= 64 && h >= 40 && (w < 3800 || h < 2100) {
                                        // Pick the most direct meaningful container
                                        if best_region.is_none() {
                                            best_region =
                                                Some(Region::new(rect.left, rect.top, w, h));
                                            // If it's already a sizeable content pane (like web body or document), return it immediately
                                            if w >= 200 && h >= 150 {
                                                CoUninitialize();
                                                return best_region;
                                            }
                                        }
                                    }
                                }

                                // Walk up to parent container
                                let tree_walker = automation.ControlViewWalker();
                                current_elem = if let Ok(walker) = tree_walker {
                                    walker.GetParentElement(&elem).ok()
                                } else {
                                    None
                                };
                            }

                            if let Some(reg) = best_region {
                                CoUninitialize();
                                return Some(reg);
                            }
                        }
                    }
                }
            }
            CoUninitialize();

            // Fallback to Win32 Top-level and Child Window enumeration
            let shell_hwnd = GetShellWindow();
            let desktop_hwnd = GetDesktopWindow();

            struct WindowSearch {
                pt: POINT,
                current_pid: u32,
                shell_hwnd: HWND,
                desktop_hwnd: HWND,
                found_hwnd: Option<HWND>,
                found_rect: Option<RECT>,
            }

            let mut search = WindowSearch {
                pt,
                current_pid,
                shell_hwnd,
                desktop_hwnd,
                found_hwnd: None,
                found_rect: None,
            };

            unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
                let search = &mut *(lparam.0 as *mut WindowSearch);

                if !IsWindowVisible(hwnd).as_bool() || IsIconic(hwnd).as_bool() {
                    return BOOL(1);
                }

                if hwnd == search.shell_hwnd || hwnd == search.desktop_hwnd {
                    return BOOL(1);
                }

                let mut pid = 0u32;
                GetWindowThreadProcessId(hwnd, Some(&mut pid));
                if pid == search.current_pid {
                    return BOOL(1);
                }

                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;
                if (ex_style & WS_EX_TRANSPARENT.0 != 0) && (ex_style & WS_EX_TOOLWINDOW.0 != 0) {
                    return BOOL(1);
                }

                let mut rect = RECT::default();
                let dwm_res = DwmGetWindowAttribute(
                    hwnd,
                    DWMWA_EXTENDED_FRAME_BOUNDS,
                    &mut rect as *mut _ as *mut _,
                    std::mem::size_of::<RECT>() as u32,
                );

                if (dwm_res.is_err()
                    || (rect.right - rect.left <= 0)
                    || (rect.bottom - rect.top <= 0))
                    && GetWindowRect(hwnd, &mut rect).is_err()
                {
                    return BOOL(1);
                }

                let w = (rect.right - rect.left).max(0);
                let h = (rect.bottom - rect.top).max(0);
                if w < 16 || h < 16 {
                    return BOOL(1);
                }

                // Ignore full-desktop background cover windows
                if rect.left <= 0 && rect.top <= 0 && w >= 3800 && h >= 2100 {
                    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
                    if (style & 0x00C00000) == 0 {
                        return BOOL(1);
                    }
                }

                if search.pt.x >= rect.left
                    && search.pt.x < rect.right
                    && search.pt.y >= rect.top
                    && search.pt.y < rect.bottom
                {
                    search.found_hwnd = Some(hwnd);
                    search.found_rect = Some(rect);
                    return BOOL(0);
                }

                BOOL(1)
            }

            let _ = EnumWindows(
                Some(enum_windows_proc),
                LPARAM(&mut search as *mut _ as isize),
            );

            let final_rect = search.found_rect?;
            let width = (final_rect.right - final_rect.left).max(0) as u32;
            let height = (final_rect.bottom - final_rect.top).max(0) as u32;

            if width < 8 || height < 8 {
                return None;
            }

            Some(Region::new(final_rect.left, final_rect.top, width, height))
        }
    }

    #[cfg(not(windows))]
    pub fn detect_element_at_point(_screen_x: i32, _screen_y: i32) -> Option<Region> {
        None
    }
}
