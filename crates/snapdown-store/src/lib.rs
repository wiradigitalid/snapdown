pub use snapdown_core;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_crate_initializes() {
        let id = snapdown_core::id_from_timestamp(1_700_000_000, 0);
        assert_eq!(id.len(), 36);
    }
}
