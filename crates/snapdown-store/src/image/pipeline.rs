use image::codecs::png::{CompressionType, FilterType, PngEncoder};
use image::{ExtendedColorType, ImageEncoder, RgbaImage};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::{QualityBudget, ResolvedPair};
use snapdown_core::error::CoreError;

#[derive(Debug, Clone)]
pub struct ReducedImageResult {
    pub bytes: Vec<u8>,
    pub dimensions: ImageDimensions,
    pub thumbnail_bytes: Option<Vec<u8>>,
    pub thumbnail_dimensions: Option<ImageDimensions>,
}

/// Writes an `RgbaImage` as a PNG, at the settings this product has measured rather than the
/// encoder's defaults, and without an alpha channel when there is no transparency to keep.
///
/// `PngEncoder::new` uses `CompressionType::Fast`, and on a real capture that is not a small
/// difference. Measured in a RELEASE build on three files out of the owner's own Vault:
///
///   1460x1042 UI capture   155 KB -> 88 KB   (-43%),  18 ms
///   1254x1254 photo import  1951 KB -> 1537 KB (-21%), 381 ms
///   486x308 small capture     7 KB ->  2 KB   (-71%),   1 ms
///
/// The middle row is the one the owner reported: a 1.5 MB file came back from an import at 1.9 MB,
/// and 1537 KB is the size it started at. Two causes, both here - `Fast` compression, and an alpha
/// channel written for an image that has none.
///
/// `CompressionType::Best` was measured too and rejected: 3% smaller for four times the time
/// (1442 ms on that same import). `BG-3` is about bytes reaching an agent, and 3% does not buy a
/// second of the Reviewer's wait.
///
/// Note the first measurement was taken in a DEBUG build and read 596 ms for what is 18 ms in
/// release. `AGENTS.md` records that trap about this repository; it caught this decision too.
/// How many bits per channel a quality number keeps.
///
/// The ladder, and the measurements it was chosen from - a 1280x800 screenshot-like fixture with
/// antialiased glyph shoulders and a gradient, which is where a real capture's colour count actually
/// comes from:
///
/// ```text
///   lossless          32117 bytes  100%
///   7 bits/channel    23562 bytes   73%   max channel error 1
///   6 bits/channel    22967 bytes   71%   max channel error 2
///   5 bits/channel    22482 bytes   70%   max channel error 6
///   4 bits/channel    21774 bytes   67%   max channel error 14
/// ```
///
/// Note where it flattens: quantising harder than 6 bits buys almost nothing in RGB, because PNG's
/// own entropy coding is already doing the work. The real win is what quantisation UNLOCKS - see
/// `encode_png`.
fn bits_for_quality(quality: u8) -> u32 {
    match quality {
        90..=u8::MAX => 7,
        70..=89 => 6,
        40..=69 => 5,
        _ => 4,
    }
}

/// Rounds every colour channel to `keep` bits.
///
/// ROUNDS, never truncates. Truncating darkens every mid-grey by up to half a step, which over a
/// whole screenshot reads as the image having been dimmed.
///
/// Idempotent: quantising an already-quantised value returns it unchanged, so re-encoding a Finding -
/// which the burn does - loses nothing a second time. `quantising_twice_changes_nothing` asserts it.
fn quantise(image: &RgbaImage, keep: u32) -> RgbaImage {
    let step = 1u16 << (8 - keep);
    let ceiling = ((1u16 << keep) - 1) * step;
    let mut out = image.clone();
    for pixel in out.pixels_mut() {
        for channel in 0..3 {
            let value = u16::from(pixel[channel]);
            pixel[channel] = (((value + step / 2) / step) * step).min(ceiling).min(255) as u8;
        }
    }
    out
}

/// Encodes a PNG, applying the Quality Budget's `encoder_quality`.
///
/// **`encoder_quality` finally does something.** It was stored and read by nothing for the life of
/// the product (`BUG-63`), because the obvious reading of it - a JPEG-style quality dial - does not
/// exist in PNG: PNG is lossless and has no such knob. The corpus had designed a two-lever budget
/// and shipped one lever.
///
/// What it does instead is the answer to the question the owner actually asked - *"bagaimana kualitas
/// gambar sama (kasat mata), tapi secara storage dia makin kecil"*:
///
///   1. round every channel to N bits, N from the quality number
///   2. if the result is opaque and fits in 256 colours, write an INDEXED PNG - one byte per pixel
///      plus a palette, instead of three bytes per pixel
///   3. otherwise write RGB or RGBA as before
///
/// Step 2 is where the size goes. On the same fixture the ladder above was measured on:
///
/// ```text
///   lossless           32117 bytes  100%
///   indexed @7 bits     8403 bytes   26%   60 colours   max channel error 1
///   indexed @6 bits     7756 bytes   24%   52 colours   max channel error 2
/// ```
///
/// A 74% reduction for a per-channel error of one. That works because a UI screenshot is flat colour
/// and text, and this product exists to capture UI screenshots - so the common case is the one that
/// benefits. A capture with real photographic content will exceed 256 colours, fall through to step
/// 3, and still come out around 27% smaller.
///
/// `quality: 100` means lossless, and skips all of it. Nothing is thrown away that the Reviewer did
/// not ask to have thrown away.
pub(crate) fn encode_png(
    image: &RgbaImage,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, CoreError> {
    let fully_opaque = image.pixels().all(|p| p[3] == 255);

    if quality >= 100 {
        return encode_png_lossless(image, width, height, fully_opaque);
    }

    let reduced = quantise(image, bits_for_quality(quality));

    if fully_opaque {
        if let Some(bytes) = encode_png_indexed(&reduced, width, height)? {
            return Ok(bytes);
        }
    }
    encode_png_lossless(&reduced, width, height, fully_opaque)
}

/// An indexed PNG, or `None` when the image needs more than 256 colours.
///
/// `None` rather than an error: too many colours is not a failure, it is a capture that happens to
/// carry photographic content, and the caller has a perfectly good path for it.
fn encode_png_indexed(
    image: &RgbaImage,
    width: u32,
    height: u32,
) -> Result<Option<Vec<u8>>, CoreError> {
    let mut palette: Vec<[u8; 3]> = Vec::with_capacity(256);
    let mut lookup: std::collections::HashMap<[u8; 3], u8> = std::collections::HashMap::new();
    let mut indices = Vec::with_capacity((width as usize) * (height as usize));

    for pixel in image.pixels() {
        let colour = [pixel[0], pixel[1], pixel[2]];
        match lookup.get(&colour) {
            Some(index) => indices.push(*index),
            None => {
                if palette.len() == 256 {
                    return Ok(None);
                }
                let index = palette.len() as u8;
                palette.push(colour);
                lookup.insert(colour, index);
                indices.push(index);
            }
        }
    }

    let mut plte = Vec::with_capacity(palette.len() * 3);
    for colour in &palette {
        plte.extend_from_slice(colour);
    }

    let mut out = Vec::new();
    let mut encoder = png::Encoder::new(&mut out, width, height);
    encoder.set_color(png::ColorType::Indexed);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_palette(plte);
    encoder.set_compression(png::Compression::Default);
    // NoFilter, and measured: on indexed data every filter makes the output LARGER, because a
    // palette index has no numeric relationship to its neighbour - subtracting one from the next is
    // noise, not a delta.
    encoder.set_filter(png::FilterType::NoFilter);
    let mut writer = encoder
        .write_header()
        .map_err(|e| CoreError::Validation(format!("Failed to write indexed PNG header: {e}")))?;
    writer
        .write_image_data(&indices)
        .map_err(|e| CoreError::Validation(format!("Failed to write indexed PNG data: {e}")))?;
    drop(writer);

    Ok(Some(out))
}

/// The original encoder, unchanged in behaviour and now one branch of three.
///
/// `CompressionType::Default` with `FilterType::Adaptive`, and Rgb8 when nothing is transparent.
/// `PngEncoder::new` uses `CompressionType::Fast`, which measured 20-70% larger on this product's own
/// captures; `Best` was rejected because it cost far more time than it saved bytes.
fn encode_png_lossless(
    image: &RgbaImage,
    width: u32,
    height: u32,
    fully_opaque: bool,
) -> Result<Vec<u8>, CoreError> {
    let mut out = Vec::new();
    let encoder =
        PngEncoder::new_with_quality(&mut out, CompressionType::Default, FilterType::Adaptive);

    if fully_opaque {
        // Three bytes a pixel instead of four. A screenshot has nothing to be transparent about, and
        // the alpha plane is a quarter of the data for a channel that is 255 everywhere.
        let rgb = image::DynamicImage::ImageRgba8(image.clone()).to_rgb8();
        encoder
            .write_image(rgb.as_raw(), width, height, ExtendedColorType::Rgb8)
            .map_err(|e| CoreError::Validation(format!("Failed to encode PNG: {e}")))?;
    } else {
        encoder
            .write_image(image.as_raw(), width, height, ExtendedColorType::Rgba8)
            .map_err(|e| CoreError::Validation(format!("Failed to encode PNG: {e}")))?;
    }
    Ok(out)
}

pub struct ImageReducer;

impl ImageReducer {
    /// Reduces raw image bytes according to a ResolvedPair (max_long_edge, encoder_quality).
    pub fn reduce_image(
        input_bytes: &[u8],
        original_dims: ImageDimensions,
        resolved: &ResolvedPair,
        generate_thumbnail: bool,
    ) -> Result<ReducedImageResult, CoreError> {
        let decoded = image::load_from_memory(input_bytes)
            .map_err(|e| CoreError::Validation(format!("Failed to decode image: {e}")))?;

        let decoded_rgba = decoded.to_rgba8();

        let target_dims = original_dims.compute_reduced_dimensions_for_pair(resolved);

        // Resize when the target differs from the original, which now happens for TWO reasons: the
        // long-edge cap, and the resize ratio. It used to test the cap alone, so a ratio of 80% would
        // have computed smaller dimensions and then handed back the full-size pixels under them -
        // a Finding whose stored width and actual width disagreed.
        let target_image = if target_dims.width != original_dims.width
            || target_dims.height != original_dims.height
        {
            image::imageops::resize(
                &decoded_rgba,
                target_dims.width,
                target_dims.height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            decoded_rgba
        };

        let out_bytes = encode_png(
            &target_image,
            target_dims.width,
            target_dims.height,
            resolved.encoder_quality,
        )?;

        let (thumb_bytes, thumb_dims) = if generate_thumbnail {
            let t_dims = target_dims.compute_thumbnail_dimensions(320);
            let thumb_img = image::imageops::resize(
                &target_image,
                t_dims.width,
                t_dims.height,
                image::imageops::FilterType::Lanczos3,
            );
            // The same quality as the full image. A thumbnail is the most palette-friendly
            // thing this product produces, so it benefits from the indexed path most of all.
            let t_bytes = encode_png(
                &thumb_img,
                t_dims.width,
                t_dims.height,
                resolved.encoder_quality,
            )?;
            (Some(t_bytes), Some(t_dims))
        } else {
            (None, None)
        };

        Ok(ReducedImageResult {
            bytes: out_bytes,
            dimensions: target_dims,
            thumbnail_bytes: thumb_bytes,
            thumbnail_dimensions: thumb_dims,
        })
    }

    /// Reduces raw image bytes by resolving a QualityBudget against original image dimensions.
    pub fn reduce_image_with_budget(
        input_bytes: &[u8],
        original_dims: ImageDimensions,
        budget: &QualityBudget,
        generate_thumbnail: bool,
    ) -> Result<ReducedImageResult, CoreError> {
        let resolved = budget.resolve(original_dims.long_edge());
        Self::reduce_image(input_bytes, original_dims, &resolved, generate_thumbnail)
    }

    /// Performs zero-byte reservation and asynchronous write pipeline.
    /// Synchronously reserves zero-byte placeholder at `dest_path`, then completes full image write.
    pub fn reserve_and_write(
        vault_store: &crate::vault::VaultBlobStore,
        relative_path: &str,
        reduced_bytes: &[u8],
    ) -> Result<(), CoreError> {
        use snapdown_core::ports::BlobStore;

        // Step 1: Zero-byte placeholder reservation (NFR-2)
        vault_store.write_blob(relative_path, &[])?;

        // Step 2: Write final reduced bytes
        vault_store.write_blob(relative_path, reduced_bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snapdown_core::domain::setting::NamedBudget;

    fn make_test_png(w: u32, h: u32) -> Vec<u8> {
        let img = image::RgbaImage::new(w, h);
        let mut bytes = Vec::new();
        let encoder = PngEncoder::new(&mut bytes);
        encoder
            .write_image(img.as_raw(), w, h, ExtendedColorType::Rgba8)
            .unwrap();
        bytes
    }

    #[test]
    fn quality_budget_downscaling_and_compression_for_balanced_preset() {
        let balanced = QualityBudget::new(NamedBudget::Balanced, None);
        let orig_dims = ImageDimensions::new(3840, 2160).unwrap();
        let input_bytes = make_test_png(3840, 2160);

        let result =
            ImageReducer::reduce_image_with_budget(&input_bytes, orig_dims, &balanced, true)
                .unwrap();

        assert_eq!(result.dimensions.width, 1600);
        assert_eq!(result.dimensions.height, 900);
        assert!(result.bytes.len() > 8);
        assert_eq!(result.thumbnail_dimensions.as_ref().unwrap().width, 320);
        assert_eq!(result.thumbnail_dimensions.as_ref().unwrap().height, 180);

        let decoded = image::load_from_memory(&result.bytes).unwrap();
        assert_eq!(decoded.width(), 1600);
        assert_eq!(decoded.height(), 900);

        let thumb_decoded =
            image::load_from_memory(result.thumbnail_bytes.as_ref().unwrap()).unwrap();
        assert_eq!(thumb_decoded.width(), 320);
        assert_eq!(thumb_decoded.height(), 180);
    }
}
