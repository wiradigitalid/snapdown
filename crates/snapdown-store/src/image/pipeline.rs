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
pub(crate) fn encode_png(image: &RgbaImage, width: u32, height: u32) -> Result<Vec<u8>, CoreError> {
    let mut out = Vec::new();
    let encoder =
        PngEncoder::new_with_quality(&mut out, CompressionType::Default, FilterType::Adaptive);

    // A capture is always opaque, so this is the normal path rather than an optimisation for an
    // unusual case. An imported PNG that really has transparency keeps it.
    let opaque = image.pixels().all(|px| px.0[3] == u8::MAX);
    if opaque {
        let mut rgb = Vec::with_capacity((width as usize) * (height as usize) * 3);
        for px in image.pixels() {
            rgb.extend_from_slice(&px.0[..3]);
        }
        encoder
            .write_image(&rgb, width, height, ExtendedColorType::Rgb8)
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

        // Downscale if image exceeds max long edge, otherwise retain without upscaling (BR-40, BR-41)
        let target_image = if original_dims.long_edge() > resolved.max_long_edge {
            image::imageops::resize(
                &decoded_rgba,
                target_dims.width,
                target_dims.height,
                image::imageops::FilterType::Lanczos3,
            )
        } else {
            decoded_rgba
        };

        let out_bytes = encode_png(&target_image, target_dims.width, target_dims.height)?;

        let (thumb_bytes, thumb_dims) = if generate_thumbnail {
            let t_dims = target_dims.compute_thumbnail_dimensions(320);
            let thumb_img = image::imageops::resize(
                &target_image,
                t_dims.width,
                t_dims.height,
                image::imageops::FilterType::Lanczos3,
            );
            let t_bytes = encode_png(&thumb_img, t_dims.width, t_dims.height)?;
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
