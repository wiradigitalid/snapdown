pub mod capturer;
pub mod error;

pub use capturer::{
    CaptureTarget, MonitorCapture, MonitorRect, RegionCapturer, VirtualDesktopCapture,
};
pub use error::CaptureError;
