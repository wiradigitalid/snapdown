#![warn(clippy::disallowed_methods)]

pub mod domain;
pub mod error;
pub mod ports;
pub mod util;

pub use domain::access_key::{AccessKey, AuthResult};
pub use domain::bundle::{Bundle, BundleDetail, BundleItem};
pub use domain::finding::{Finding, FindingDetail, Marker, Note, Region};
pub use domain::image::ImageDimensions;
pub use domain::markdown::MarkdownSerializer;
pub use domain::publication::Publication;
pub use domain::setting::{QualityBudget, Setting, SettingKey, SettingValue};
pub use error::CoreError;
pub use ports::{
    AccessKeyStore, BlobStore, BundleStore, Clock, EntropySource, FindingStore, PublicationStore,
    SettingsStore,
};
pub use util::id::id_from_parts;
