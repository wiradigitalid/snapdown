//! Every annotation shape reaches the burned PNG - `FR-30`, `FR-31`, `FR-32`.
//!
//! `BUG-72`'s plan said the burn "already exists and is tested; it only needs to be handed real
//! data". Two of the five shapes disproved that: `AnnotationShape::Text` fell into a `_ => {}` and
//! was never drawn at all, and a Callout's `text` was discarded by a `..` so the bubble burned empty.
//! Five passing burner tests covered shapes the burner would never be handed, and none of them
//! looked for a letter.
//!
//! Every assertion here DECODES the output, per `AGENTS.md`: "an image test that asserts a signature
//! and a dimension is a test that a fake header passes".

use image::codecs::png::PngEncoder;
use image::{ExtendedColorType, ImageEncoder, Rgba, RgbaImage};
use snapdown_core::domain::finding::{AnnotationShape, VisualAnnotation};
use snapdown_core::domain::image::ImageDimensions;
use snapdown_store::image::MarkerBurner;

const W: u32 = 400;
const H: u32 = 300;

fn white_png() -> Vec<u8> {
    let img = RgbaImage::from_pixel(W, H, Rgba([255, 255, 255, 255]));
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(img.as_raw(), W, H, ExtendedColorType::Rgba8)
        .expect("encode fixture");
    bytes
}

/// Alternating columns, so a blur has something to average away and the test can see it happen.
fn striped_png() -> Vec<u8> {
    let mut img = RgbaImage::new(W, H);
    for y in 0..H {
        for x in 0..W {
            let v = if x % 2 == 0 { 0u8 } else { 255u8 };
            img.put_pixel(x, y, Rgba([v, v, v, 255]));
        }
    }
    let mut bytes = Vec::new();
    PngEncoder::new(&mut bytes)
        .write_image(img.as_raw(), W, H, ExtendedColorType::Rgba8)
        .expect("encode fixture");
    bytes
}

fn ann(id: &str, data: AnnotationShape) -> VisualAnnotation {
    VisualAnnotation {
        id: id.to_string(),
        finding_id: "f-1".to_string(),
        data,
        created_at: "2026-08-28T10:00:00Z".to_string(),
    }
}

fn burn(source: &[u8], annotations: &[VisualAnnotation]) -> RgbaImage {
    let dims = ImageDimensions::new(W, H).unwrap();
    let bytes = MarkerBurner::burn_all(source, &dims, &[], annotations).expect("burn must succeed");
    image::load_from_memory(&bytes)
        .expect("the burned output must be a real, decodable PNG")
        .to_rgba8()
}

/// Counts pixels inside a normalized box that are not the given colour.
fn ink_in(img: &RgbaImage, x: f64, y: f64, w: f64, h: f64, ground: Rgba<u8>) -> u32 {
    let x0 = (x * W as f64) as u32;
    let y0 = (y * H as f64) as u32;
    let x1 = ((x + w) * W as f64).min(W as f64) as u32;
    let y1 = ((y + h) * H as f64).min(H as f64) as u32;
    let mut count = 0;
    for py in y0..y1 {
        for px in x0..x1 {
            if *img.get_pixel(px, py) != ground {
                count += 1;
            }
        }
    }
    count
}

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);

#[test]
fn a_text_annotation_puts_letters_on_the_image() {
    let img = burn(
        &white_png(),
        &[ann(
            "a-text",
            AnnotationShape::Text {
                x: 0.1,
                y: 0.1,
                width: 0.6,
                height: 0.2,
                text: "Hello".to_string(),
                font_size: Some(28.0),
                font_family: None,
                text_color: Some("#000000".to_string()),
                text_align: None,
            },
        )],
    );

    let ink = ink_in(&img, 0.1, 0.1, 0.6, 0.2, WHITE);
    assert!(
        ink > 50,
        "a Text annotation must draw its letters. Found {ink} non-white pixels in its box - this \
         arm did not exist, so the answer used to be 0"
    );

    // And nowhere else. A Text annotation has no border and no fill; it is only the words.
    let elsewhere = ink_in(&img, 0.0, 0.5, 1.0, 0.5, WHITE);
    assert_eq!(
        elsewhere, 0,
        "the letters must stay inside the annotation's own box"
    );
}

#[test]
fn a_callout_draws_a_filled_rounded_bubble_a_solid_tail_and_white_words() {
    let img = burn(
        &white_png(),
        &[ann(
            "a-callout",
            AnnotationShape::Callout {
                x: 0.1,
                y: 0.1,
                width: 0.6,
                height: 0.3,
                tail_x: 0.5,
                tail_y: 0.8,
                text: "the save button is disabled".to_string(),
                font_size: Some(20.0),
                font_family: None,
                bg_color: None,
                text_color: None,
                text_align: None,
            },
        )],
    );

    let red = Rgba([220, 38, 38, 255]);

    assert_eq!(
        *img.get_pixel(20, 40),
        WHITE,
        "outside the bubble the image is untouched"
    );

    // FILLED, not outlined. A Callout is a LABEL, and a label needs a ground of its own to be read
    // against - which is also what lets its words be white while the other four shapes draw in red.
    assert_eq!(
        *img.get_pixel((0.65 * W as f64) as u32, (0.36 * H as f64) as u32),
        red,
        "a Callout must be filled. It was an outline, and the owner asked for the fill back"
    );

    // ROUNDED. The exact corner pixel is outside the shape; a point just inside the same corner box
    // on the diagonal is inside it.
    let x0 = (0.1 * W as f64) as u32;
    let y0 = (0.1 * H as f64) as u32;
    assert_eq!(
        *img.get_pixel(x0, y0),
        WHITE,
        "the corner must be rounded - a square corner would paint this pixel"
    );
    assert_eq!(
        *img.get_pixel(x0 + 9, y0 + 9),
        red,
        "and just inside that corner the plate is solid"
    );

    // AN ARROW, not a triangle, reaching the point it names.
    //
    // The target is below the bubble, so the shaft leaves the bottom edge at the x nearest the
    // target - here `tail_x` itself, 0.5 - and runs straight down to it. Sampled on that line,
    // halfway between the bubble and the target.
    assert_eq!(
        *img.get_pixel((0.5 * W as f64) as u32, (0.6 * H as f64) as u32),
        red,
        "the callout's tail must be an ARROW leaving the bubble edge nearest its target"
    );

    // And it has a head at the target end. The chevron opens backwards from the tip, so a point
    // just up and to the side of the tip is on it.
    let tip_x = (0.5 * W as f64) as u32;
    let tip_y = (0.8 * H as f64) as u32;
    let mut head = 0;
    for py in (tip_y - 18)..=tip_y {
        for px in (tip_x - 18)..=(tip_x + 18) {
            if *img.get_pixel(px, py) == red {
                head += 1;
            }
        }
    }
    assert!(
        head > 60,
        "the arrow must have a head at the point it names. Found {head} red pixels around the tip -          a bare line would give only the shaft's own width"
    );

    // And the words, in WHITE, on the plate.
    let mut lettering = 0;
    for py in (0.12 * H as f64) as u32..(0.38 * H as f64) as u32 {
        for px in (0.12 * W as f64) as u32..(0.68 * W as f64) as u32 {
            if *img.get_pixel(px, py) == WHITE {
                lettering += 1;
            }
        }
    }
    assert!(
        lettering > 50,
        "a Callout's words must be white ON the plate. Found {lettering} white pixels inside it - \
         red words on a red fill would be invisible, which is why this one shape differs"
    );
}

/// The one that matters most: a redaction that does not reach the file leaves the password on it.
#[test]
fn a_blur_annotation_actually_changes_the_pixels_under_it() {
    let source = striped_png();
    let before = image::load_from_memory(&source).unwrap().to_rgba8();
    let img = burn(
        &source,
        &[ann(
            "a-blur",
            AnnotationShape::Blur {
                x: 0.25,
                y: 0.25,
                width: 0.5,
                height: 0.5,
                blur_radius: None,
            },
        )],
    );

    let inside_x = (0.5 * W as f64) as u32;
    let inside_y = (0.5 * H as f64) as u32;
    assert_ne!(
        img.get_pixel(inside_x, inside_y),
        before.get_pixel(inside_x, inside_y),
        "the pixels under a redaction box must be destroyed in the burned file"
    );

    // The stripes are black/white - amplitude 255. After the blur the local swing must have
    // collapsed, or whatever was written there is still legible.
    let mut lo = 255u8;
    let mut hi = 0u8;
    for px in inside_x..(inside_x + 20) {
        let v = img.get_pixel(px, inside_y)[0];
        lo = lo.min(v);
        hi = hi.max(v);
    }
    assert!(
        hi - lo < 20,
        "the redaction must flatten the contrast it covers - a 20px run still swings by {} of 255",
        hi - lo
    );

    // And outside it nothing moved.
    assert_eq!(
        img.get_pixel(5, 5),
        before.get_pixel(5, 5),
        "a redaction must not touch the rest of the screenshot"
    );
    // Right up to the edge: the window is clamped to the box, so a pixel just outside is untouched.
    let edge = (0.25 * W as f64) as u32 - 1;
    assert_eq!(
        img.get_pixel(edge, inside_y),
        before.get_pixel(edge, inside_y),
        "and it must stop exactly at the box - a blur that bled outward would soften the screenshot \
         around every redaction"
    );
}

/// A mosaic is not a blur, and this is the difference.
///
/// The first implementation averaged each `radius`-sized BLOCK to one flat colour and called it a
/// blur. Over a gradient that produces visible steps - runs of identical pixels as long as the
/// block. A real blur is a moving average, so every pixel gets its own value and a gradient stays
/// smooth. The owner reported the steps: *"maunya blurnya jangan mosaik begitu"*.
#[test]
fn a_blur_is_a_blur_and_not_a_mosaic() {
    // A horizontal gradient, which a mosaic turns into stairs and a blur leaves as a gradient.
    let mut fixture = RgbaImage::new(W, H);
    for y in 0..H {
        for x in 0..W {
            let v = (x * 255 / W) as u8;
            fixture.put_pixel(x, y, Rgba([v, v, v, 255]));
        }
    }
    let mut source = Vec::new();
    PngEncoder::new(&mut source)
        .write_image(fixture.as_raw(), W, H, ExtendedColorType::Rgba8)
        .expect("encode gradient");

    let img = burn(
        &source,
        &[ann(
            "a-blur",
            AnnotationShape::Blur {
                x: 0.2,
                y: 0.2,
                width: 0.6,
                height: 0.6,
                blur_radius: Some(16.0),
            },
        )],
    );

    let y = (0.5 * H as f64) as u32;
    let x0 = (0.25 * W as f64) as u32;
    let x1 = (0.75 * W as f64) as u32;

    let mut longest_run = 1;
    let mut run = 1;
    for x in (x0 + 1)..x1 {
        if img.get_pixel(x, y)[0] == img.get_pixel(x - 1, y)[0] {
            run += 1;
            longest_run = longest_run.max(run);
        } else {
            run = 1;
        }
    }

    assert!(
        longest_run < 8,
        "a blur of a gradient stays a gradient. Found a run of {longest_run} identical pixels, which \
         is a mosaic block - the very thing this replaced"
    );

    // And it is still a blur, not a copy: the gradient is smooth, so it must be SMOOTHER than the
    // source at its own scale rather than untouched. Checked at the box edge, where a blur clamped
    // to the region flattens hardest.
    assert_ne!(
        img.get_pixel(x0, y),
        fixture.get_pixel(x0, y),
        "the region must actually have been processed"
    );
}

#[test]
fn a_shape_and_an_arrow_reach_the_file() {
    let img = burn(
        &white_png(),
        &[
            ann(
                "a-rect",
                AnnotationShape::Rect {
                    x: 0.05,
                    y: 0.05,
                    width: 0.3,
                    height: 0.3,
                    stroke_color: None,
                    stroke_width: Some(4.0),
                },
            ),
            ann(
                "a-arrow",
                AnnotationShape::Arrow {
                    start_x: 0.6,
                    start_y: 0.2,
                    end_x: 0.9,
                    end_y: 0.8,
                    color: None,
                    stroke_width: Some(4.0),
                },
            ),
        ],
    );

    assert!(
        ink_in(&img, 0.05, 0.05, 0.3, 0.3, WHITE) > 20,
        "the Shape must be drawn"
    );
    assert!(
        ink_in(&img, 0.6, 0.2, 0.3, 0.6, WHITE) > 20,
        "the Arrow must be drawn"
    );

    // A Shape is an OUTLINE, per `FR-30`: "transparent fill with theme-accent outline". Its middle
    // must still show the screenshot.
    assert_eq!(
        *img.get_pixel((0.2 * W as f64) as u32, (0.2 * H as f64) as u32),
        WHITE,
        "a Shape must not fill - the Reviewer is boxing something in order to still see it"
    );
}

/// `FR-32`'s alignment control, which the owner asked for as "justify page".
#[test]
fn text_alignment_moves_the_words_within_the_box() {
    let make = |align: &str| {
        burn(
            &white_png(),
            &[ann(
                "a",
                AnnotationShape::Text {
                    x: 0.0,
                    y: 0.1,
                    width: 1.0,
                    height: 0.2,
                    text: "abc".to_string(),
                    font_size: Some(30.0),
                    font_family: None,
                    text_color: Some("#000000".to_string()),
                    text_align: Some(align.to_string()),
                },
            )],
        )
    };

    let start = make("start");
    let end = make("end");

    let left_half = |img: &RgbaImage| ink_in(img, 0.0, 0.1, 0.4, 0.2, WHITE);
    let right_half = |img: &RgbaImage| ink_in(img, 0.6, 0.1, 0.4, 0.2, WHITE);

    assert!(
        left_half(&start) > 0 && right_half(&start) == 0,
        "start-aligned text belongs on the left"
    );
    assert!(
        right_half(&end) > 0 && left_half(&end) == 0,
        "end-aligned text belongs on the right"
    );
}

/// Z-order is the order the store returns, and the store returns creation order.
///
/// Checked on a BORDER pixel, because nothing fills any more: two boxes sharing an edge, and the one
/// drawn second owns it.
#[test]
fn a_later_annotation_covers_an_earlier_one() {
    let boxed = |colour: &str| AnnotationShape::Rect {
        x: 0.2,
        y: 0.2,
        width: 0.4,
        height: 0.4,
        stroke_color: Some(colour.to_string()),
        stroke_width: Some(6.0),
    };

    let img = burn(
        &white_png(),
        &[
            ann("first", boxed("#ff0000")),
            ann("second", boxed("#00ff00")),
        ],
    );

    assert_eq!(
        *img.get_pixel((0.4 * W as f64) as u32, (0.2 * H as f64) as u32),
        Rgba([0, 255, 0, 255]),
        "the annotation drawn second must be on top - that is what the Reviewer saw on the canvas"
    );
}

/// `AD-9`'s byte identity, re-checked now that annotations can exist.
#[test]
fn a_finding_with_no_markers_and_no_annotations_is_copied_byte_for_byte() {
    let source = white_png();
    let dims = ImageDimensions::new(W, H).unwrap();
    let out = MarkerBurner::burn_all(&source, &dims, &[], &[]).unwrap();
    assert_eq!(
        out, source,
        "with nothing to draw the Bundle's copy is the Finding's own bytes"
    );
}

/// Red is the default ink for every annotation, `Text` included.
///
/// It was white - invisible on a white capture, which is what the owner saw: *"Font dari element
/// Text masih putih, harusnya semua merah default warna dari element"*.
#[test]
fn text_with_no_colour_of_its_own_is_drawn_in_the_annotation_red() {
    let img = burn(
        &white_png(),
        &[ann(
            "a-text",
            AnnotationShape::Text {
                x: 0.1,
                y: 0.1,
                width: 0.6,
                height: 0.2,
                text: "Hello".to_string(),
                font_size: Some(28.0),
                font_family: None,
                text_color: None,
                text_align: None,
            },
        )],
    );

    let red = Rgba([220, 38, 38, 255]);
    let mut ink = 0;
    for py in (0.1 * H as f64) as u32..(0.3 * H as f64) as u32 {
        for px in (0.1 * W as f64) as u32..(0.7 * W as f64) as u32 {
            if *img.get_pixel(px, py) == red {
                ink += 1;
            }
        }
    }
    assert!(
        ink > 30,
        "unstyled Text must burn in #dc2626. Found {ink} red pixels - white text on a white capture          is text nobody can see"
    );
}

/// The default blur strength follows the region, and stays inside the band it promises.
///
/// It was a constant, twice: 8 inherited from a MOSAIC BLOCK SIZE, which barely softened text, then
/// 18, which flattened a large area to one smear. The owner asked for the middle - *"lebih soft
/// mewakili apa yang dibelakangnya"* - and the honest answer to "how much" is "it depends how big
/// the region is".
#[test]
fn a_blur_with_no_radius_of_its_own_scales_with_the_region() {
    let source = striped_png();

    let blur_at = |x: f64, y: f64, w: f64, h: f64| {
        let img = burn(
            &source,
            &[ann(
                "b",
                AnnotationShape::Blur {
                    x,
                    y,
                    width: w,
                    height: h,
                    blur_radius: None,
                },
            )],
        );
        // How flat the region came out: the swing over a 20px run at its centre.
        let cy = ((y + h / 2.0) * H as f64) as u32;
        let cx = ((x + w / 2.0) * W as f64) as u32;
        let mut lo = 255u8;
        let mut hi = 0u8;
        for px in cx..(cx + 20).min(W - 1) {
            let v = img.get_pixel(px, cy)[0];
            lo = lo.min(v);
            hi = hi.max(v);
        }
        hi - lo
    };

    // A big region gets a big radius and comes out flat.
    assert!(
        blur_at(0.1, 0.1, 0.8, 0.8) < 20,
        "a large region must be blurred enough to destroy the stripes under it"
    );

    // A thin strip gets a small radius - and must still visibly blur, or a redaction over one line
    // of text would do nothing at all.
    let thin = blur_at(0.2, 0.45, 0.6, 0.06);
    assert!(
        thin < 200,
        "even a thin strip must be blurred: the stripes swing 255, and this came out at {thin}"
    );
}
