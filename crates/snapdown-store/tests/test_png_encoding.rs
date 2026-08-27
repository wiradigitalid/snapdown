//! How this product writes a PNG, asserted rather than assumed.
//!
//! The owner reported a 1.5 MB image coming back from an import at 1.9 MB. Two causes, both in
//! `ImageReducer`: `PngEncoder::new` uses `CompressionType::Fast`, and every image was written with
//! an alpha channel whether it had transparency or not. A capture never does.
//!
//! These are behaviour tests, not size-literal tests. A test that pinned "98 KB" would break on the
//! next `image` release and tell nobody anything; each of these compares the product's output against
//! the alternative it is supposed to beat, on the same input.

use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ColorType, ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::ResolvedPair;
use snapdown_store::image::ImageReducer;

/// Noisy enough that compression settings actually differ on it. A flat fill compresses to nothing
/// under every setting, which would make the comparison below pass by accident.
fn noisy(width: u32, height: u32, opaque: bool) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    let mut seed: u32 = 0x2545_F491;
    for px in img.pixels_mut() {
        // xorshift, so the fixture is identical on every machine - no Rng, no seed to drift.
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        let b = seed.to_le_bytes();
        *px = Rgba([b[0], b[1], b[2], if opaque { 255 } else { b[3] | 0x0F }]);
    }
    img
}

fn as_png(img: &RgbaImage, ct: CompressionType, ft: FilterType) -> Vec<u8> {
    let mut out = Vec::new();
    PngEncoder::new_with_quality(&mut out, ct, ft)
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgba8,
        )
        .expect("the fixture encoder must succeed");
    out
}

/// Runs a source image through the real reduction path, with a budget that does not resize it.
fn through_reducer(img: &RgbaImage) -> Vec<u8> {
    let source = as_png(img, CompressionType::Default, FilterType::Adaptive);
    let dims = ImageDimensions {
        width: img.width(),
        height: img.height(),
    };
    let resolved = ResolvedPair {
        max_long_edge: img.width().max(img.height()),
        encoder_quality: 90,
    };
    ImageReducer::reduce_image(&source, dims, &resolved, false)
        .expect("reduction must succeed")
        .bytes
}

#[test]
fn an_opaque_image_is_written_without_an_alpha_channel() {
    let out = through_reducer(&noisy(160, 120, true));
    let decoded = image::load_from_memory(&out).expect("output must decode");
    assert_eq!(
        decoded.color(),
        ColorType::Rgb8,
        "a fully opaque image must be written as RGB. Writing the alpha channel of an image that \
         has none is a quarter of the raw bytes for no information at all, and a capture is always \
         opaque"
    );
}

#[test]
fn an_image_with_transparency_keeps_its_alpha() {
    let out = through_reducer(&noisy(160, 120, false));
    let decoded = image::load_from_memory(&out).expect("output must decode");
    assert_eq!(
        decoded.color(),
        ColorType::Rgba8,
        "dropping alpha must depend on the image, not on the format. An imported PNG that really \
         has transparency keeps it"
    );
}

/// The same payload, at `PngEncoder::new`'s compression, so ONLY the compression setting differs.
///
/// The first version of this test compared against RGBA bytes and a mutant that reverted the
/// compression to `Fast` SURVIVED it - dropping alpha alone still made the product's output smaller,
/// so the assertion was measuring the other fix. Two changes landed together and one test cannot
/// hold both ends.
fn as_png_rgb(img: &RgbaImage, ct: CompressionType, ft: FilterType) -> Vec<u8> {
    let mut rgb = Vec::with_capacity((img.width() as usize) * (img.height() as usize) * 3);
    for px in img.pixels() {
        rgb.extend_from_slice(&px.0[..3]);
    }
    let mut out = Vec::new();
    PngEncoder::new_with_quality(&mut out, ct, ft)
        .write_image(&rgb, img.width(), img.height(), ExtendedColorType::Rgb8)
        .expect("the fixture encoder must succeed");
    out
}

/// A fixture shaped like what this product actually captures: flat panels, hard edges, hairlines.
///
/// The compression test first used the random fixture above and FAILED by 20 bytes - random data is
/// incompressible, so `Default` spends more effort than `Fast` for a hair more overhead and loses.
/// That was an honest result about the wrong input: a screenshot is mostly flat regions, and
/// structure is the whole reason a compression setting matters. Measured on the owner's own captures,
/// the real gap is 43%.
fn screenshot_like(width: u32, height: u32) -> RgbaImage {
    let mut img = RgbaImage::new(width, height);
    for (x, y, px) in img.enumerate_pixels_mut() {
        let band = (y / 24) % 3;
        let mut c = match band {
            0 => [28, 28, 31],
            1 => [241, 245, 249],
            _ => [35, 35, 38],
        };
        // A panel, and a 1px rule down its left edge.
        if x > width / 4 && x < width * 3 / 4 && y > height / 3 && y < height * 2 / 3 {
            c = [255, 255, 255];
        }
        if x == width / 4 + 1 || y % 24 == 0 {
            c = [203, 213, 225];
        }
        *px = Rgba([c[0], c[1], c[2], 255]);
    }
    img
}

#[test]
fn the_encoder_beats_the_crate_default_compression() {
    let img = screenshot_like(320, 240);
    let ours = through_reducer(&img);
    let at_crate_default = as_png_rgb(&img, CompressionType::Fast, FilterType::Adaptive);

    assert!(
        ours.len() < at_crate_default.len(),
        "the product's compression setting must beat `PngEncoder::new`'s on the same payload: \
         ours {} bytes, Fast {} bytes. BG-3 is about how few bytes reach an agent",
        ours.len(),
        at_crate_default.len()
    );
}

#[test]
fn dropping_the_alpha_channel_is_worth_bytes_on_its_own() {
    let img = noisy(320, 240, true);
    let ours = through_reducer(&img);
    let with_alpha = as_png(&img, CompressionType::Default, FilterType::Adaptive);

    assert!(
        ours.len() < with_alpha.len(),
        "writing an opaque image as RGB must beat writing it as RGBA at the same compression: \
         ours {} bytes, RGBA {} bytes",
        ours.len(),
        with_alpha.len()
    );
}

/// `CompressionType::Best` was measured and rejected. This records the trade so it is not quietly
/// re-litigated: it was 3% smaller for four times the time on a real 1254x1254 import.
#[test]
fn the_encoder_does_not_pay_four_times_the_time_for_three_percent() {
    // Comment lines stripped: `encode_png`'s own doc comment names `Best` to say why it was
    // rejected, and a guard satisfied by prose that merely mentions a token is no guard.
    let source: String = include_str!("../src/image/pipeline.rs")
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        source.contains("CompressionType::Default"),
        "the encoder must use CompressionType::Default"
    );
    assert!(
        !source.contains("CompressionType::Best"),
        "CompressionType::Best was measured at 1442 ms against Default's 381 ms on the owner's own \
         1254x1254 import, for 3% fewer bytes. If that trade is being revisited, re-measure in a \
         RELEASE build - the debug figures are 15x higher and led this decision astray once already"
    );
}
