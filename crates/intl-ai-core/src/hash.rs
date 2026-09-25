use sha1::{Digest, Sha1};

/// SHA-1 lowercase hex of the UTF-8 flat source value (plan 5.1.2). Staleness
/// is stored-vs-current comparison of this digest.
pub fn source_hash(value: &str) -> String {
    let digest = Sha1::digest(value.as_bytes());
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha1_of_known_vector() {
        // sha1("Hello") is a stable published test vector.
        assert_eq!(
            source_hash("Hello"),
            "f7ff9e8b7bb2e09b70935a5d785e0cc5d9d0abf0"
        );
    }
}
