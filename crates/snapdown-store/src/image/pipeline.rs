use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder};
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

        let mut out_bytes = Vec::new();
        let encoder = PngEncoder::new(&mut out_bytes);
        encoder
            .write_image(
                target_image.as_raw(),
                target_dims.width,
                target_dims.height,
                ExtendedColorType::Rgba8,
            )
            .map_err(|e| CoreError::Validation(format!("Failed to encode PNG: {e}")))?;

        let (thumb_bytes, thumb_dims) = if generate_thumbnail {
            let t_dims = target_dims.compute_thumbnail_dimensions(320);
            let thumb_img = image::imageops::resize(
                &target_image,
                t_dims.width,
                t_dims.height,
                image::imageops::FilterType::Lanczos3,
            );
            let mut t_bytes = Vec::new();
            let thumb_encoder = PngEncoder::new(&mut t_bytes);
            thumb_encoder
                .write_image(
                    thumb_img.as_raw(),
                    t_dims.width,
                    t_dims.height,
                    ExtendedColorType::Rgba8,
                )
                .map_err(|e| {
                    CoreError::Validation(format!("Failed to encode thumbnail PNG: {e}"))
                })?;
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
