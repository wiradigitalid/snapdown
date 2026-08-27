//! BUG-28: the virtual-desktop canvas must be written exactly ONCE, straight into the buffer that
//! will be presented.
//!
//! It used to be built twice and then blended: `RgbaImage::new` for a canvas, then
//! `image::imageops::overlay` per monitor - which is a per-pixel `get_pixel`/`blend`/`put_pixel`
//! loop, not a blit - and then a full copy into the toolkit's own buffer. Measured in release on a
//! 6000x3840 two-monitor desktop: 75-81ms for the blend and 19ms for the copy, out of ~220ms
//! total. Both are pure waste, because every source pixel is opaque and the destination is
//! untouched, so the blend's result is byte-for-byte a copy.
//!
//! `identical_bytes_to_the_stitch_it_replaces` is the guard that keeps this equivalence honest.

use image::{Rgba, RgbaImage};
use snapdown_capture::{CaptureError, MonitorCapture, MonitorRect, VirtualDesktopCapture};

/// A monitor whose every pixel differs from every other, in both axes, and differs between
/// monitors through `tag`.
///
/// A flat fill will not do, and that is not a style preference: it was tried first, and mutation
/// testing showed two mutants surviving because of it. With one colour per monitor every row is
/// byte-identical to every other, so shifting the blit's source by a whole row changes nothing
/// any assertion can see. The row index is in the green channel and the column index in the red
/// for exactly that reason.
fn monitor(origin_x: i32, origin_y: i32, width: u32, height: u32, tag: u8) -> MonitorCapture {
    let mut image = RgbaImage::new(width, height);
    for (x, y, pixel) in image.enumerate_pixels_mut() {
        *pixel = Rgba([(x + 1) as u8, (y + 1) as u8, tag, 255]);
    }
    MonitorCapture {
        image,
        width,
        height,
        origin_x,
        origin_y,
        scale_factor: 1.0,
        name: format!("TEST-{origin_x},{origin_y}"),
    }
}

/// Builds the capture the way `capture_virtual_desktop` does, from monitors that are already
/// grabbed - so these tests need no display.
fn desktop(captures: Vec<MonitorCapture>) -> VirtualDesktopCapture {
    let origin_x = captures.iter().map(|c| c.origin_x).min().unwrap();
    let origin_y = captures.iter().map(|c| c.origin_y).min().unwrap();
    let max_x = captures
        .iter()
        .map(|c| c.origin_x + c.width as i32)
        .max()
        .unwrap();
    let max_y = captures
        .iter()
        .map(|c| c.origin_y + c.height as i32)
        .max()
        .unwrap();
    let monitors = captures
        .iter()
        .map(|c| MonitorRect {
            x: c.origin_x - origin_x,
            y: c.origin_y - origin_y,
            width: c.width,
            height: c.height,
            name: c.name.clone(),
        })
        .collect();
    VirtualDesktopCapture {
        captures,
        width: (max_x - origin_x) as u32,
        height: (max_y - origin_y) as u32,
        origin_x,
        origin_y,
        monitors,
    }
}

fn pixel_at(buffer: &[u8], canvas_width: u32, x: u32, y: u32) -> [u8; 4] {
    let at = ((y * canvas_width + x) * 4) as usize;
    [buffer[at], buffer[at + 1], buffer[at + 2], buffer[at + 3]]
}

/// What [`monitor`] wrote at its own local `(x, y)`. Naming the source pixel this way keeps each
/// assertion about *which* pixel landed where, not merely about a colour being present.
fn source_pixel(x: u32, y: u32, tag: u8) -> [u8; 4] {
    [(x + 1) as u8, (y + 1) as u8, tag, 255]
}

const LANDSCAPE: u8 = 10;
const PORTRAIT: u8 = 20;

/// The layout that matters on the machine this was found on: a landscape monitor at the origin and
/// a taller portrait one placed left of and above it, so the canvas is larger than either and part
/// of it is covered by no monitor at all.
fn mixed_layout() -> Vec<MonitorCapture> {
    vec![
        monitor(0, 0, 40, 20, LANDSCAPE),
        monitor(-20, -8, 20, 40, PORTRAIT),
    ]
}

#[test]
fn each_monitor_lands_at_its_own_offset_in_the_canvas() {
    let desktop = desktop(mixed_layout());
    assert_eq!((desktop.width, desktop.height), (60, 40));
    assert_eq!((desktop.origin_x, desktop.origin_y), (-20, -8));

    let mut buffer = vec![0u8; desktop.byte_len()];
    desktop.blit_into(&mut buffer).expect("blit must succeed");

    // The portrait monitor starts at the canvas origin: canvas (0,0) is its own (0,0).
    assert_eq!(pixel_at(&buffer, 60, 0, 0), source_pixel(0, 0, PORTRAIT));
    assert_eq!(
        pixel_at(&buffer, 60, 19, 39),
        source_pixel(19, 39, PORTRAIT)
    );
    // The landscape monitor sits 20 right and 8 down from it, so canvas (20,8) is its own (0,0).
    assert_eq!(pixel_at(&buffer, 60, 20, 8), source_pixel(0, 0, LANDSCAPE));
    assert_eq!(
        pixel_at(&buffer, 60, 59, 27),
        source_pixel(39, 19, LANDSCAPE)
    );
    // A row in the middle, which is what catches a blit whose rows are off by one.
    assert_eq!(pixel_at(&buffer, 60, 25, 15), source_pixel(5, 7, LANDSCAPE));
}

#[test]
fn a_gap_no_monitor_covers_is_left_untouched() {
    let desktop = desktop(mixed_layout());
    let mut buffer = vec![0xAAu8; desktop.byte_len()];
    desktop.blit_into(&mut buffer).expect("blit must succeed");

    // Bottom-right of the canvas: right of the portrait monitor, below the landscape one.
    // Nothing covers it, so the caller's own fill must still be there.
    assert_eq!(
        pixel_at(&buffer, 60, 59, 39),
        [0xAA, 0xAA, 0xAA, 0xAA],
        "the blit must write only where a monitor actually is - the caller owns what a gap holds"
    );
}

/// The equivalence this whole change rests on. `imageops::overlay` alpha-blends every pixel; a row
/// copy does not. For opaque sources over an untouched destination the results are identical, and
/// this is what proves it rather than asserting it.
#[test]
fn identical_bytes_to_the_stitch_it_replaces() {
    let desktop = desktop(mixed_layout());

    let mut reference = RgbaImage::new(desktop.width, desktop.height);
    for capture in &desktop.captures {
        image::imageops::overlay(
            &mut reference,
            &capture.image,
            (capture.origin_x - desktop.origin_x) as i64,
            (capture.origin_y - desktop.origin_y) as i64,
        );
    }

    let mut buffer = vec![0u8; desktop.byte_len()];
    desktop.blit_into(&mut buffer).expect("blit must succeed");

    assert_eq!(
        buffer,
        *reference.as_raw(),
        "the row blit must be byte-identical to the imageops::overlay stitch it replaces"
    );
}

/// A monitor exactly as wide as the canvas is one contiguous run, which is the single-monitor case
/// every user with one display hits. It must still produce the same bytes as the general path.
#[test]
fn a_full_width_monitor_is_blitted_as_one_run() {
    let desktop = desktop(vec![monitor(0, 0, 32, 24, LANDSCAPE)]);
    assert_eq!((desktop.width, desktop.height), (32, 24));

    let mut buffer = vec![0u8; desktop.byte_len()];
    desktop.blit_into(&mut buffer).expect("blit must succeed");

    assert_eq!(buffer, *desktop.captures[0].image.as_raw());
}

/// Two full-width monitors stacked vertically both take the contiguous path, and the lower one
/// takes it at a non-zero row offset. Without this the fast path was only ever exercised at the top
/// of the canvas, so dropping its row offset altogether changed nothing any test could see - a
/// surviving mutant, found by mutating it.
#[test]
fn a_stacked_full_width_monitor_is_blitted_at_its_own_row_offset() {
    let desktop = desktop(vec![
        monitor(0, 0, 32, 24, LANDSCAPE),
        monitor(0, 24, 32, 16, PORTRAIT),
    ]);
    assert_eq!((desktop.width, desktop.height), (32, 40));

    let mut buffer = vec![0u8; desktop.byte_len()];
    desktop.blit_into(&mut buffer).expect("blit must succeed");

    assert_eq!(pixel_at(&buffer, 32, 0, 0), source_pixel(0, 0, LANDSCAPE));
    assert_eq!(
        pixel_at(&buffer, 32, 31, 23),
        source_pixel(31, 23, LANDSCAPE)
    );
    // The second monitor's own (0,0) must land on canvas row 24, not on row 0.
    assert_eq!(pixel_at(&buffer, 32, 0, 24), source_pixel(0, 0, PORTRAIT));
    assert_eq!(
        pixel_at(&buffer, 32, 31, 39),
        source_pixel(31, 15, PORTRAIT)
    );
}

#[test]
fn a_buffer_of_the_wrong_size_is_an_error_not_a_panic() {
    let desktop = desktop(mixed_layout());

    let mut too_small = vec![0u8; desktop.byte_len() - 4];
    assert!(matches!(
        desktop.blit_into(&mut too_small),
        Err(CaptureError::InvalidRegion(_))
    ));

    let mut too_large = vec![0u8; desktop.byte_len() + 4];
    assert!(matches!(
        desktop.blit_into(&mut too_large),
        Err(CaptureError::InvalidRegion(_))
    ));
}

/// A display can be unplugged between the enumeration that sized the canvas and the grab. The blit
/// must drop such a monitor rather than index outside the buffer: a panic here takes the tray, the
/// hotkeys and the Editor with it, because they all live in this one process.
#[test]
fn a_monitor_that_does_not_fit_the_canvas_is_skipped_not_panicked_on() {
    let mut desktop = desktop(vec![monitor(0, 0, 40, 20, LANDSCAPE)]);
    desktop.captures.push(monitor(30, 10, 40, 20, PORTRAIT));

    let mut buffer = vec![0u8; desktop.byte_len()];
    desktop
        .blit_into(&mut buffer)
        .expect("an ill-fitting monitor must not fail the whole blit");

    // The monitor that does fit is still there in full.
    assert_eq!(pixel_at(&buffer, 40, 0, 0), source_pixel(0, 0, LANDSCAPE));
    assert_eq!(
        pixel_at(&buffer, 40, 39, 19),
        source_pixel(39, 19, LANDSCAPE)
    );
}
