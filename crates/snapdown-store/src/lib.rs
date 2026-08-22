pub use snapdown_core;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_crate_initializes() {
        let rand_b = [0u8; 10];
        let id = snapdown_core::id_from_parts(1_700_000_000_000, rand_b);
        assert_eq!(id.len(), 36);
    }
}
