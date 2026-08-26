use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_capture::capturer::RegionCapturer;
use snapdown_core::domain::finding::{Marker, Region};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_core::domain::setting::ResolvedPair;
use snapdown_store::image::{ImageReducer, MarkerBurner};

fn make_gradient_image(w: u32, h: u32) -> RgbaImage {
    let mut img = RgbaImage::new(w, h);
    for y in 0..h {
        for x in 0..w {
            let r = ((x * 255) / w.max(1)) as u8;
            let g = ((y * 255) / h.max(1)) as u8;
            let b = (((x + y) * 128) / (w + h).max(1)) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    img
}

fn encode_png(img: &RgbaImage) -> Vec<u8> {
    let mut bytes = Vec::new();
    let encoder = PngEncoder::new(&mut bytes);
    encoder
        .write_image(
            img.as_raw(),
            img.width(),
            img.height(),
            ExtendedColorType::Rgba8,
        )
        .expect("Failed to encode test PNG fixture");
    bytes
}

fn assert_not_uniform_fill(img: &RgbaImage, producer_name: &str) {
    let (w, h) = (img.width(), img.height());
    assert!(w > 0 && h > 0, "{producer_name}: Image has 0 dimension");

    let first_pixel = img.get_pixel(0, 0);
    let mut found_differing = false;

    for y in 0..h {
        for x in 0..w {
            if img.get_pixel(x, y) != first_pixel {
                found_differing = true;
                break;
            }
        }
        if found_differing {
            break;
        }
    }

    assert!(
        found_differing,
        "{producer_name}: Decoded image output is a uniform fill; expected non-uniform image content"
    );
}

/// Covers:
/// - Positive obligation driving all three image producers:
///   1. RegionCapturer::crop_and_encode_image
///   2. ImageReducer::reduce_image
///   3. MarkerBurner::burn_markers
/// - For each producer:
///   - Output decodes cleanly with image::load_from_memory
///   - Output matches expected dimensions
///   - Decoded pixels are NOT a uniform solid fill
#[test]
fn every_image_producing_path_decodes_its_own_output() {
    // ------------------------------------------------------------------------
    // Producer 1: RegionCapturer::crop_and_encode_image
    // ------------------------------------------------------------------------
    {
        let src_img = make_gradient_image(800, 600);
        let crop_region = Region {
            x: 50,
            y: 50,
            width: 400,
            height: 300,
        };

        let cropped_bytes = RegionCapturer::crop_and_encode_image(&src_img, &crop_region)
            .expect("RegionCapturer::crop_and_encode_image must succeed");

        let decoded = image::load_from_memory(&cropped_bytes)
            .expect("RegionCapturer output must decode cleanly as PNG")
            .to_rgba8();

        assert_eq!(
            decoded.width(),
            400,
            "RegionCapturer decoded width must match requested region width"
        );
        assert_eq!(
            decoded.height(),
            300,
            "RegionCapturer decoded height must match requested region height"
        );
        assert_not_uniform_fill(&decoded, "RegionCapturer");
    }

    // ------------------------------------------------------------------------
    // Producer 2: ImageReducer::reduce_image
    // ------------------------------------------------------------------------
    {
        let src_img = make_gradient_image(1600, 1200);
        let src_bytes = encode_png(&src_img);
        let orig_dims = ImageDimensions::new(1600, 1200).unwrap();
        let pair = ResolvedPair::new(800, 80).unwrap();

        let reduced = ImageReducer::reduce_image(&src_bytes, orig_dims, &pair, false)
            .expect("ImageReducer::reduce_image must succeed");

        let decoded = image::load_from_memory(&reduced.bytes)
            .expect("ImageReducer output must decode cleanly as PNG")
            .to_rgba8();

        assert_eq!(
            decoded.width(),
            800,
            "ImageReducer decoded width must match reduced dimensions width"
        );
        assert_eq!(
            decoded.height(),
            600,
            "ImageReducer decoded height must match reduced dimensions height"
        );
        assert_eq!(reduced.dimensions.width, 800);
        assert_eq!(reduced.dimensions.height, 600);
        assert_not_uniform_fill(&decoded, "ImageReducer");
    }

    // ------------------------------------------------------------------------
    // Producer 3: MarkerBurner::burn_markers
    // ------------------------------------------------------------------------
    {
        let src_img = make_gradient_image(500, 400);
        let src_bytes = encode_png(&src_img);
        let dims = ImageDimensions::new(500, 400).unwrap();
        let marker = Marker::new(
            "m1".into(),
            "f1".into(),
            1,
            0.5,
            0.5,
            "Annotation note text".into(),
        )
        .unwrap();

        let burned_bytes = MarkerBurner::burn_markers(&src_bytes, &dims, &[marker])
            .expect("MarkerBurner::burn_markers must succeed");

        let decoded = image::load_from_memory(&burned_bytes)
            .expect("MarkerBurner output must decode cleanly as PNG")
            .to_rgba8();

        assert_eq!(
            decoded.width(),
            500,
            "MarkerBurner decoded width must match source dimensions width"
        );
        assert_eq!(
            decoded.height(),
            400,
            "MarkerBurner decoded height must match source dimensions height"
        );
        assert_not_uniform_fill(&decoded, "MarkerBurner");
    }
}
