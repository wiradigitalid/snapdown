use crate::domain::publication::Publication;
use crate::error::CoreError;

pub trait PublicationStore: Send + Sync {
    fn get_by_bundle_id(&self, bundle_id: &str) -> Result<Option<Publication>, CoreError>;
    fn get_by_slug(&self, slug: &str) -> Result<Option<Publication>, CoreError>;
    fn save(&self, publication: &Publication) -> Result<(), CoreError>;
    fn mark_unpublished(&self, bundle_id: &str, unpublished_at: &str) -> Result<(), CoreError>;
    fn set_last_error(&self, bundle_id: &str, error: Option<&str>) -> Result<(), CoreError>;
    fn delete_by_bundle_id(&self, bundle_id: &str) -> Result<(), CoreError>;
    fn list_active(&self) -> Result<Vec<Publication>, CoreError>;
}
