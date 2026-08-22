use uuid::Uuid;

pub fn new_id() -> String {
    Uuid::now_v7().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_lowercase_hyphenated_uuidv7() {
        let id1 = new_id();
        let id2 = new_id();

        assert_eq!(id1.len(), 36);
        assert_eq!(id1, id1.to_lowercase());
        assert_ne!(id1, id2);

        let parsed = Uuid::parse_str(&id1).expect("valid uuid");
        assert_eq!(parsed.get_version_num(), 7);
    }
}
