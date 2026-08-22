pub fn id_from_parts(unix_millis: u64, rand_b: [u8; 10]) -> String {
    let mut bytes = [0u8; 16];

    // 48-bit timestamp in milliseconds (big-endian)
    bytes[0] = (unix_millis >> 40) as u8;
    bytes[1] = (unix_millis >> 32) as u8;
    bytes[2] = (unix_millis >> 24) as u8;
    bytes[3] = (unix_millis >> 16) as u8;
    bytes[4] = (unix_millis >> 8) as u8;
    bytes[5] = unix_millis as u8;

    // 12-bit rand_a / sub-millis with ver = 7 in high 4 bits (0x70..0x7F)
    // We fill bytes 6 and 7 using the first 2 bytes of rand_b with version 7 masked in
    let ver_and_rand = 0x7000u16 | (((rand_b[0] as u16) << 8 | (rand_b[1] as u16)) & 0x0FFF);
    bytes[6] = (ver_and_rand >> 8) as u8;
    bytes[7] = ver_and_rand as u8;

    // RFC 9562 / RFC 4122 variant (0b10xxxxxx) in high 2 bits of byte 8
    bytes[8] = 0x80 | (rand_b[2] & 0x3F);

    // Remaining random bytes (rand_b[3..10] into bytes[9..16])
    bytes[9] = rand_b[3];
    bytes[10] = rand_b[4];
    bytes[11] = rand_b[5];
    bytes[12] = rand_b[6];
    bytes[13] = rand_b[7];
    bytes[14] = rand_b[8];
    bytes[15] = rand_b[9];

    format!(
        "{:02x}{:02x}{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        bytes[0],
        bytes[1],
        bytes[2],
        bytes[3],
        bytes[4],
        bytes[5],
        bytes[6],
        bytes[7],
        bytes[8],
        bytes[9],
        bytes[10],
        bytes[11],
        bytes[12],
        bytes[13],
        bytes[14],
        bytes[15]
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_lowercase_hyphenated_uuidv7_from_parts() {
        let rand1 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
        let rand2 = [1, 2, 3, 4, 5, 6, 7, 8, 9, 11];
        let id1 = id_from_parts(1_700_000_000_000, rand1);
        let id2 = id_from_parts(1_700_000_000_000, rand2);

        assert_eq!(id1.len(), 36);
        assert_eq!(id1, id1.to_lowercase());
        assert_ne!(id1, id2);

        // Verify UUID structure and format: 8-4-4-4-12 hex characters
        let parts: Vec<&str> = id1.split('-').collect();
        assert_eq!(parts.len(), 5);
        assert_eq!(parts[0].len(), 8);
        assert_eq!(parts[1].len(), 4);
        assert_eq!(parts[2].len(), 4);
        assert_eq!(parts[3].len(), 4);
        assert_eq!(parts[4].len(), 12);

        // Version nibble (first nibble of 3rd group) must be '7'
        assert!(parts[2].starts_with('7'));

        // Variant nibble (first nibble of 4th group) must be 8, 9, a, or b (0b10xxxxxx)
        let var_char = parts[3].chars().next().unwrap();
        assert!(
            var_char == '8' || var_char == '9' || var_char == 'a' || var_char == 'b',
            "Variant character must be 8, 9, a, or b, got: {var_char}"
        );
    }
}
