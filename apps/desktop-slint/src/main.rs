#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use snapdown_capture::RegionCapturer;

slint::include_modules!();

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let main_window = AppWindow::new()?;

    // Native Window Dragging on Titlebar
    #[cfg(windows)]
    {
        main_window.on_drag_window_requested(|| {
            use windows::Win32::UI::Input::KeyboardAndMouse::ReleaseCapture;
            use windows::Win32::UI::WindowsAndMessaging::{
                GetForegroundWindow, SendMessageW, HTCAPTION, WM_NCLBUTTONDOWN,
            };
            unsafe {
                let hwnd = GetForegroundWindow();
                if !hwnd.is_invalid() {
                    let _ = ReleaseCapture();
                    let _ = SendMessageW(
                        hwnd,
                        WM_NCLBUTTONDOWN,
                        Some(windows::Win32::Foundation::WPARAM(HTCAPTION as usize)),
                        Some(windows::Win32::Foundation::LPARAM(0)),
                    );
                }
            }
        });
    }

    // Window controls
    let win_min = main_window.as_weak();
    main_window.on_minimize_clicked(move || {
        if let Some(win) = win_min.upgrade() {
            win.window().set_minimized(true);
        }
    });

    let win_max = main_window.as_weak();
    main_window.on_maximize_clicked(move || {
        if let Some(win) = win_max.upgrade() {
            let is_max = win.window().is_maximized();
            win.window().set_maximized(!is_max);
        }
    });

    let win_close = main_window.as_weak();
    main_window.on_close_clicked(move || {
        if let Some(win) = win_close.upgrade() {
            win.hide().unwrap();
            std::process::exit(0);
        }
    });

    // Theme Toggle
    main_window.on_theme_toggle_clicked(|| {
        println!("Toggle Dark/Light theme clicked");
    });

    // Bundles Drawer Toggle
    main_window.on_bundles_drawer_clicked(|| {
        println!("Toggle Bundles Drawer clicked");
    });

    // Settings Toggle
    main_window.on_settings_clicked(|| {
        println!("Open Settings clicked");
    });

    // Setup Capture callback
    let window_weak = main_window.as_weak();
    main_window.on_capture_clicked(move || {
        let (img, w, h) = match RegionCapturer::capture_monitor_image(None) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Capture failed: {e}");
                return;
            }
        };

        // Convert raw RgbaImage to Slint RgbaImage
        let pixel_buffer = slint::SharedPixelBuffer::<slint::Rgba8Pixel>::clone_from_slice(
            img.as_raw(),
            w,
            h,
        );
        let slint_img = slint::Image::from_rgba8(pixel_buffer);

        let overlay = CaptureOverlayWindow::new().unwrap();
        overlay.set_snapshot_image(slint_img.clone());

        let overlay_weak = overlay.as_weak();
        let main_weak = window_weak.clone();

        overlay.on_capture_completed(move |_x, _y, w, h, note| {
            if let Some(main) = main_weak.upgrade() {
                main.set_active_image(slint_img.clone());
                main.set_resolution_text(format!("{} × {} px", w, h).into());
                main.set_size_text(format!("{:.1} KB", (w * h * 4) as f64 / 1024.0 / 6.0).into());
                if !note.is_empty() {
                    main.set_observation_summary(note);
                }
                main.show().unwrap();
            }
            if let Some(ov) = overlay_weak.upgrade() {
                ov.hide().unwrap();
            }
        });

        let overlay_cancel = overlay.as_weak();
        let main_restore = window_weak.clone();
        overlay.on_overlay_cancelled(move || {
            if let Some(ov) = overlay_cancel.upgrade() {
                ov.hide().unwrap();
            }
            if let Some(main) = main_restore.upgrade() {
                main.show().unwrap();
            }
        });

        overlay.show().unwrap();
    });

    main_window.on_open_file_clicked(|| {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Images", &["png", "jpg", "jpeg", "webp"])
            .pick_file()
        {
            println!("Selected file: {:?}", path);
        }
    });

    main_window.run()?;
    Ok(())
}
