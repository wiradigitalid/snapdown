use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CaptureError {
    #[error("No display found for screen capture")]
    NoDisplayFound,

    #[error("Monitor not found: {0}")]
    MonitorNotFound(String),

    #[error("Requested region {requested} exceeds monitor bounds {monitor}")]
    RegionExceedsMonitorBounds { requested: String, monitor: String },

    #[error("Invalid region: {0}")]
    InvalidRegion(String),

    #[error("Screen capture failed: {0}")]
    CaptureFailed(String),

    #[error("Image encoding failed: {0}")]
    EncodingFailed(String),
}
