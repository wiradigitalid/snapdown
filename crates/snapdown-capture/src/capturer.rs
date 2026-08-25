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
