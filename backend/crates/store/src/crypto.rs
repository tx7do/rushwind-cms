//! Credential crypto primitives: the AES-128-CBC front-end password
//! layer (key `f51d66a73d8a0927`, IV = the key itself, PKCS#7), bcrypt
//! hashing (default cost 12), and the timing-equalizer dummy verify.

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};

type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

/// The default AES key — 16 bytes, also reused as the IV (the
/// reference's crypto.DefaultAESKey).
pub const DEFAULT_AES_KEY: &[u8; 16] = b"f51d66a73d8a0927";

/// Decrypts the login credential: base64(AES-128-CBC(key, iv=key, PKCS7)).
pub fn decrypt_login_credential(plain_credential: &str) -> Result<String, String> {
    use base64::Engine as _;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(plain_credential.trim())
        .map_err(|e| format!("credential base64: {e}"))?;
    decrypt_aes_cbc(&bytes).ok_or_else(|| "credential decrypt".to_string())
}

/// AES-128-CBC decrypt with the DefaultAESKey pair (bytes in).
pub fn decrypt_aes_cbc(ciphertext: &[u8]) -> Option<String> {
    let mut out = vec![0u8; ciphertext.len()];
    let len = Aes128CbcDec::new(DEFAULT_AES_KEY.into(), DEFAULT_AES_KEY.into())
        .decrypt_padded_b2b_mut::<Pkcs7>(ciphertext, &mut out)
        .ok()?
        .len();
    out.truncate(len);
    String::from_utf8(out).ok()
}

/// bcrypt hash at default cost.
pub fn hash_password(password: &str) -> Result<String, String> {
    bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| format!("bcrypt: {e}"))
}

/// bcrypt verify — the raw `$2a$…` stored string.
pub fn verify_password(password: &str, hash: &str) -> bool {
    bcrypt::verify(password, hash).unwrap_or(false)
}

/// The timing-equalizer dummy hash — the user-not-found paths run one
/// verification against it so response times do not leak existence. It
/// rides the same cost as `hash_password`'s output: the equalizer only
/// balances when the dummy verification burns the same work as the real
/// one. Its plaintext ("rushwind-timing-equalizer-placeholder") is no
/// real credential, so the probe below never matches.
pub const DUMMY_PASSWORD_HASH: &str =
    "$2b$12$pjz1dJ6s0Ix9Q3Dh246ZRu9tHdOssjsiV4H.kgaKUgtSwDGe4JeC2";

pub fn dummy_verify() {
    let _ = bcrypt::verify("definitely-not-the-password", DUMMY_PASSWORD_HASH);
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;

    #[test]
    fn aes_cbc_reference_vector() {
        // openssl enc -aes-128-cbc of "s3cret" with the reference key/iv —
        // the decrypt direction pins the padding and the key pair.
        let ct = unb64("h1EAUNtwVm48wiz7iW1sjw==");
        assert_eq!(decrypt_aes_cbc(&ct).as_deref(), Some("admin"));
    }

    /// The encrypt direction of the front-end layer — fixtures for the
    /// decrypt pins (the library only carries the decrypt half).
    fn encrypt_login_credential(plain: &[u8]) -> Vec<u8> {
        use aes::cipher::BlockEncryptMut;
        type Aes128CbcEnc = cbc::Encryptor<aes::Aes128>;
        let mut out = vec![0u8; plain.len() + 16];
        let len = Aes128CbcEnc::new(DEFAULT_AES_KEY.into(), DEFAULT_AES_KEY.into())
            .encrypt_padded_b2b_mut::<Pkcs7>(plain, &mut out)
            .unwrap()
            .len();
        out.truncate(len);
        out
    }

    fn b64(bytes: &[u8]) -> String {
        base64::engine::general_purpose::STANDARD.encode(bytes)
    }

    fn unb64(text: &str) -> Vec<u8> {
        base64::engine::general_purpose::STANDARD
            .decode(text)
            .unwrap()
    }

    #[test]
    fn aes_cbc_round_trips_every_plaintext_shape() {
        // A partial block, an exact block, an exact block plus one byte
        // (a whole padding block), and multi-byte UTF-8 across the block
        // boundary.
        for plain in [
            "a",
            "admin",
            "0123456789abcdef",
            "0123456789abcdef0",
            "pässwörd-密码-🔑",
        ] {
            let ct = encrypt_login_credential(plain.as_bytes());
            assert_eq!(decrypt_aes_cbc(&ct).as_deref(), Some(plain), "{plain}");
        }
    }

    #[test]
    fn aes_cbc_rejects_malformed_ciphertext() {
        // Not block-aligned, or empty: nothing to decrypt.
        assert_eq!(decrypt_aes_cbc(&[0u8; 5]), None);
        assert_eq!(decrypt_aes_cbc(&[]), None);

        // Aligned but the padding byte is out of range: a two-block
        // message truncated to its data blocks ends in 0x41.
        let ct = encrypt_login_credential(&[0x41; 32]);
        assert_eq!(decrypt_aes_cbc(&ct[..32]), None);
        // ...or zero.
        let mut msg = vec![0x41; 32];
        msg[31] = 0;
        let ct = encrypt_login_credential(&msg);
        assert_eq!(decrypt_aes_cbc(&ct[..32]), None);

        // Valid padding, but the plaintext is not UTF-8.
        let ct = encrypt_login_credential(&[0xff; 15]);
        assert_eq!(decrypt_aes_cbc(&ct), None);
    }

    #[test]
    fn login_credential_unwraps_base64_and_tolerates_padding() {
        // The reference vector, as the wire carries it.
        assert_eq!(
            decrypt_login_credential("h1EAUNtwVm48wiz7iW1sjw==").unwrap(),
            "admin"
        );
        // Surrounding whitespace (a form field's stray newline) is trimmed.
        assert_eq!(
            decrypt_login_credential("  h1EAUNtwVm48wiz7iW1sjw==\n").unwrap(),
            "admin"
        );
        // A round trip through the front-end's own encryption.
        let wire = b64(&encrypt_login_credential("s3cret 密码".as_bytes()));
        assert_eq!(decrypt_login_credential(&wire).unwrap(), "s3cret 密码");

        // Not base64 at all → the base64 stage reports.
        let err = decrypt_login_credential("not base64!").unwrap_err();
        assert!(err.starts_with("credential base64:"), "{err}");
        // Base64 of bytes the cipher refuses → the decrypt stage reports.
        let err = decrypt_login_credential(&b64(&[0u8; 7])).unwrap_err();
        assert_eq!(err, "credential decrypt");
    }

    #[test]
    fn hash_and_verify_round_trip() {
        let hash = hash_password("s3cret!").unwrap();
        assert!(hash.starts_with("$2"), "{hash}");
        assert_ne!(hash, "s3cret!", "the plaintext never rides the stored form");
        assert!(verify_password("s3cret!", &hash));
        assert!(!verify_password("s3cret", &hash));
        assert!(!verify_password("", &hash));
        // Every call salts afresh: the same password never stores twice alike.
        assert_ne!(hash_password("s3cret!").unwrap(), hash);
    }

    #[test]
    fn verify_never_panics_on_a_malformed_stored_hash() {
        for junk in ["", "not-a-hash", "$2a$10$tooshort", "$2z$10$nope"] {
            assert!(!verify_password("s3cret!", junk), "{junk:?}");
            assert!(!verify_password("", junk), "{junk:?}");
        }
    }

    #[test]
    fn dummy_verify_is_a_live_hash_that_never_accepts() {
        // The equalizer must really run a bcrypt verification — a constant
        // that no longer parses would short-circuit it into a no-op.
        assert!(
            !bcrypt::verify("definitely-not-the-password", DUMMY_PASSWORD_HASH).unwrap(),
            "the dummy constant must stay a parsable bcrypt hash"
        );
        assert!(!verify_password(
            "definitely-not-the-password",
            DUMMY_PASSWORD_HASH
        ));
        assert!(!verify_password("", DUMMY_PASSWORD_HASH));
        // And the probe itself is infallible.
        dummy_verify();
    }

    /// The bcrypt cost segment off a `$2?$$NN$…` stored string.
    fn cost_of(hash: &str) -> Option<u32> {
        hash.split('$').nth(2)?.parse().ok()
    }

    #[test]
    fn the_dummy_equalizer_rides_the_real_hashing_cost() {
        // A cheaper dummy would answer user-not-found faster than a real
        // password check — the response time would leak existence again.
        // hash_password rides the crate's DEFAULT_COST; the dummy must too.
        let real = cost_of(&hash_password("cost probe").unwrap()).unwrap();
        assert_eq!(real, bcrypt::DEFAULT_COST, "hash_password's cost drifted");
        assert_eq!(
            cost_of(DUMMY_PASSWORD_HASH),
            Some(bcrypt::DEFAULT_COST),
            "the dummy equalizer must match the real hashing cost"
        );
    }
}
