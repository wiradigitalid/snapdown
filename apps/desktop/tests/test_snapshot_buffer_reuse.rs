//! The capture canvas is reused across captures instead of being reallocated, because
//! `SharedPixelBuffer::new` is not a `calloc` - it fills through `SharedVector`'s per-element
//! `FromIterator` push loop, so a 6000x3840 buffer costs 33-37ms while a plain 92 MB write pass
//! costs 3.6ms (`BUG-28`).
//!
//! Reuse rests on `make_mut_bytes` writing in place, which it only does while nothing else holds
//! the buffer. That is what the tests below pin down.
//!
//! **What they deliberately do NOT claim.** An earlier version of this file asserted that clearing
//! the overlay's `snapshot-image` property is enough to make the canvas uniquely owned again. That
//! test passed, and it was wrong about the running application: Slint bindings are lazy, so the two
//! elements that bind the image - the backdrop and the loupe - latch their `source` when they
//! RENDER and keep it until they render again. A hidden window never renders, so between captures
//! they still hold the last image shown, and clearing the root property does not reach them. The
//! test passed only because a component constructed in a test harness has never rendered, so its
//! bindings had never latched anything.
//!
//! It was caught by the canary in `on_capture_clicked`, which compares the buffer's data pointer
//! across the write and reported a copy on all 28 captures of a session. The design that followed
//! does not depend on releasing anything: two canvases are written alternately, so whatever the
//! elements still hold is the previous capture's, and the one being written is free by
//! construction. A test that cannot render cannot prove that - the canary can, and does.

use slint::{Rgba8Pixel, SharedPixelBuffer};

fn data_ptr(buffer: &SharedPixelBuffer<Rgba8Pixel>) -> *const u8 {
    buffer.as_bytes().as_ptr()
}

/// What alternating buys: the canvas being written is held by nothing, so the write is in place.
#[test]
fn a_uniquely_owned_buffer_is_written_in_place() {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(64, 48);
    let before = data_ptr(&buffer);

    buffer.make_mut_bytes().fill(0x5A);

    assert!(
        std::ptr::eq(before, data_ptr(&buffer)),
        "writing a buffer nobody else holds must not move it - if it does, every capture pays a \
         92 MB copy and reusing the canvas buys nothing (BUG-28)"
    );
    assert!(buffer.as_bytes().iter().all(|b| *b == 0x5A));
}

/// The failure mode, reproduced: correct pixels, silently at the cost of a 92 MB copy. Nothing
/// about the resulting image differs, which is why the capture path checks a pointer instead.
#[test]
fn a_buffer_with_a_live_clone_is_copied_before_it_is_written() {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(64, 48);
    buffer.make_mut_bytes().fill(0x11);
    let before = data_ptr(&buffer);

    let held_elsewhere = buffer.clone();

    buffer.make_mut_bytes().fill(0x22);

    assert!(
        !std::ptr::eq(before, data_ptr(&buffer)),
        "a shared buffer must be copied before it is written - if it were not, the capture path's \
         pointer comparison would be reading a coincidence (BUG-28)"
    );
    assert!(
        held_elsewhere.as_bytes().iter().all(|b| *b == 0x11),
        "the clone must keep the pixels it was made from"
    );
    assert!(
        buffer.as_bytes().iter().all(|b| *b == 0x22),
        "and the writer must see its own write"
    );
}

/// Handing a buffer to a Slint `Image` shares it. This is the half of the story that IS true at the
/// component level, and it is why the canvas being written must be the one that was NOT handed over
/// last time rather than the one that was.
#[test]
fn wrapping_a_buffer_in_an_image_shares_it() {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(64, 48);
    buffer.make_mut_bytes().fill(0x33);
    let before = data_ptr(&buffer);

    let shown = slint::Image::from_rgba8(buffer.clone());

    buffer.make_mut_bytes().fill(0x44);

    assert!(
        !std::ptr::eq(before, data_ptr(&buffer)),
        "an Image made from a buffer must hold that buffer, or the alternation the capture path \
         relies on would be solving a problem that does not exist (BUG-28)"
    );
    assert_eq!(
        shown.size().width,
        64,
        "the Image keeps its own view of the pixels"
    );
}

/// Dropping the last other owner restores writing in place. This is what makes ALTERNATION work:
/// by the time a canvas comes round again, the `Image` that held it has been replaced.
#[test]
fn releasing_the_last_other_owner_restores_writing_in_place() {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(64, 48);
    let held_elsewhere = buffer.clone();
    drop(held_elsewhere);

    let before = data_ptr(&buffer);
    buffer.make_mut_bytes().fill(0x55);

    assert!(
        std::ptr::eq(before, data_ptr(&buffer)),
        "dropping the last other owner must make the buffer uniquely owned again, or a canvas \
         would never become writable in place no matter how many are alternated (BUG-28)"
    );
}
