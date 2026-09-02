use crate::domain::bundle::{Bundle, BundleDetail, BundleItem};
use crate::error::CoreError;

pub trait BundleStore {
    fn create_bundle(&self, bundle: &Bundle, items: &[BundleItem]) -> Result<(), CoreError>;

    fn get_bundle(&self, id: &str) -> Result<Option<BundleDetail>, CoreError>;

    fn list_bundles(&self) -> Result<Vec<BundleDetail>, CoreError>;

    /// Writes a Bundle's name and its document TOGETHER, under `BR-5`: Review & Update's Save
    /// (ticket 14) is the one caller, and the two never make sense apart - the title lives in both
    /// the row's `name` column and the document's own `# ` heading, and a write that landed one
    /// without the other would leave the Library and the document disagreeing about the Bundle's own
    /// name. Subsumes the document-only `update_bundle_markdown` that ticket 05's audit found nothing
    /// called.
    ///
    /// `updated_at` moves the Bundle's last-edited time (ticket 15) to exactly the instant the
    /// caller names - the same `Clock`-supplied-string shape `FindingStore::update_note` already
    /// uses, rather than this method reaching for its own clock. The ONE caller
    /// (`save_review_update_edit`) only ever calls this method on the branch that already decided a
    /// real change exists, so "moves only when the update actually writes" holds by construction:
    /// there is no separate no-op guard in here to keep in sync with that one.
    fn update_bundle_name_and_markdown(
        &self,
        id: &str,
        name: &str,
        markdown: &str,
        updated_at: &str,
    ) -> Result<(), CoreError>;

    fn delete_bundle(&self, id: &str) -> Result<(), CoreError>;
}
