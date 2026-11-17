//! Lowercase hex, as every PhyUDM hash, key id and signature is written.

use std::fmt::Write as _;

/// A 32-byte SHA-256 hash.
pub type Hash = [u8; 32];

/// Lowercase hex of `bytes`.
pub fn encode(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        // Writing to a String cannot fail.
        let _ = write!(out, "{b:02x}");
    }
    out
}

/// Bytes of a hex string (either case); `None` if it is not hex.
pub fn decode(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 {
        return None;
    }
    s.as_bytes()
        .chunks(2)
        .map(|pair| {
            let digit = |b: u8| char::from(b).to_digit(16);
            let hi = digit(*pair.first()?)?;
            let lo = digit(*pair.get(1)?)?;
            u8::try_from(hi * 16 + lo).ok()
        })
        .collect()
}

/// A 64-character hex string as a [`Hash`].
pub fn hash(s: &str) -> Option<Hash> {
    if s.len() != 64 {
        return None;
    }
    Hash::try_from(decode(s)?).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_and_rejects_bad_input() {
        assert_eq!(encode(&[0, 15, 255]), "000fff");
        assert_eq!(decode("000fFF"), Some(vec![0, 15, 255]));
        assert_eq!(decode("abc"), None);
        assert_eq!(decode("zz"), None);
        assert_eq!(hash("00"), None);
        assert_eq!(hash(&"ab".repeat(32)), Some([0xab; 32]));
    }
}
