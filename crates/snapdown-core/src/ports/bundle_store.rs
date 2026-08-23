use crate::domain::bundle::{Bundle, BundleDetail, BundleItem};
use crate::error::CoreError;

pub trait BundleStore {
    fn create_bundle(&self, bundle: &Bundle, items: &[BundleItem]) -> Result<(), CoreError>;

    fn get_bundle(&self, id: &str) -> Result<Option<BundleDetail>, CoreError>;

    fn list_bundles(&self) -> Result<Vec<BundleDetail>, CoreError>;

    fn update_bundle_markdown(&self, id: &str, markdown: &str) -> Result<(), CoreError>;

    fn delete_bundle(&self, id: &str) -> Result<(), CoreError>;
}
