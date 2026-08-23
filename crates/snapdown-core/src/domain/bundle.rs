use serde::{Deserialize, Serialize};

use crate::error::CoreError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bundle {
    pub id: String,
    pub name: String,
    pub markdown: String,
    pub markdown_path: String,
    pub composed_at: String,
}

impl Bundle {
    pub fn new(
        id: String,
        name: String,
        markdown: String,
        markdown_path: String,
        composed_at: String,
    ) -> Result<Self, CoreError> {
        if name.trim().is_empty() {
            return Err(CoreError::Validation("Bundle name cannot be empty".into()));
        }
        Ok(Self {
            id,
            name,
            markdown,
            markdown_path,
            composed_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleItem {
    pub id: String,
    pub bundle_id: String,
    pub finding_id: String,
    pub position: u32,
    pub image_path: String,
}

impl BundleItem {
    pub fn new(
        id: String,
        bundle_id: String,
        finding_id: String,
        position: u32,
        image_path: String,
    ) -> Result<Self, CoreError> {
        if position == 0 {
            return Err(CoreError::Validation(
                "BundleItem position must be >= 1".into(),
            ));
        }
        Ok(Self {
            id,
            bundle_id,
            finding_id,
            position,
            image_path,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BundleDetail {
    pub bundle: Bundle,
    pub items: Vec<BundleItem>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundle_creation_and_validation() {
        assert!(Bundle::new(
            "b1".into(),
            "".into(),
            "# Review".into(),
            "bundles/b1.md".into(),
            "2026-08-23T10:00:00Z".into()
        )
        .is_err());

        let bundle = Bundle::new(
            "b1".into(),
            "Release 1.0 Review".into(),
            "# Review".into(),
            "bundles/b1.md".into(),
            "2026-08-23T10:00:00Z".into(),
        )
        .unwrap();

        assert_eq!(bundle.name, "Release 1.0 Review");
    }

    #[test]
    fn bundle_item_validation() {
        assert!(BundleItem::new(
            "bi1".into(),
            "b1".into(),
            "f1".into(),
            0,
            "bundles/b1/f1.png".into()
        )
        .is_err());

        let item = BundleItem::new(
            "bi1".into(),
            "b1".into(),
            "f1".into(),
            1,
            "bundles/b1/f1.png".into(),
        )
        .unwrap();

        assert_eq!(item.position, 1);
    }
}
