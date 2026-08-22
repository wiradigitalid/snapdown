use uuid::Uuid;

pub fn id_from_timestamp(seconds: u64, nanos: u32) -> String {
    let millis = seconds
        .saturating_mul(1000)
        .saturating_add((nanos / 1_000_000) as u64);
    let sub_millis_fract = (nanos % 1_000_000) as u16;
    let mut bytes = [0u8; 16];

    // 48-bit timestamp in milliseconds (big-endian)
    bytes[0] = (millis >> 40) as u8;
    bytes[1] = (millis >> 32) as u8;
    bytes[2] = (millis >> 24) as u8;
    bytes[3] = (millis >> 16) as u8;
    bytes[4] = (millis >> 8) as u8;
    bytes[5] = millis as u8;

    // 12-bit sub-millisecond precision / sequence (ver = 7)
    let ver_and_sub = 0x7000u16 | (sub_millis_fract & 0x0FFF);
    bytes[6] = (ver_and_sub >> 8) as u8;
    bytes[7] = ver_and_sub as u8;

    // RFC 4122 variant (0b10xxxxxx)
    bytes[8] = 0x80;
    // Remainder can be deterministic or 0
    bytes[9] = 0x00;
    bytes[10] = 0x00;
    bytes[11] = 0x00;
    bytes[12] = 0x00;
    bytes[13] = 0x00;
    bytes[14] = 0x00;
    bytes[15] = 0x01;

    Uuid::from_bytes(bytes).hyphenated().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_valid_lowercase_hyphenated_uuidv7_from_timestamp() {
        let id1 = id_from_timestamp(1_700_000_000, 100);
        let id2 = id_from_timestamp(1_700_000_000, 200);

        assert_eq!(id1.len(), 36);
        assert_eq!(id1, id1.to_lowercase());
        assert_ne!(id1, id2);

        let parsed = Uuid::parse_str(&id1).expect("valid uuid");
        assert_eq!(parsed.get_version_num(), 7);
    }
}
