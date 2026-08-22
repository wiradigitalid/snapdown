pub use snapdown_core;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_crate_initializes() {
        let id = snapdown_core::new_id();
        assert_eq!(id.len(), 36);
    }
}
