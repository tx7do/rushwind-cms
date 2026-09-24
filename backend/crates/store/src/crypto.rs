//! Credential crypto primitives: the AES-128-CBC front-end password
//! layer (key `f51d66a73d8a0927`, IV = the key itself, PKCS#7), bcrypt
//! hashing (default cost 10), and the timing-equalizer dummy verify.

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
/// verification against it so response times do not leak existence.
pub const DUMMY_PASSWORD_HASH: &str =
    "$2a$10$1sbpKmhQDpXLHnDnEQ1nLe3oOnYyP2bUJyqHcX2T0Fq1qfyoXOrPm";

pub fn dummy_verify() {
    let _ = bcrypt::verify("definitely-not-the-password", DUMMY_PASSWORD_HASH);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aes_cbc_reference_vector() {
        // openssl enc -aes-128-cbc of "s3cret" with the reference key/iv —
        // the decrypt direction pins the padding and the key pair.
        use base64::Engine as _;
        let ct = base64::engine::general_purpose::STANDARD
            .decode("h1EAUNtwVm48wiz7iW1sjw==")
            .unwrap();
        assert_eq!(decrypt_aes_cbc(&ct).as_deref(), Some("admin"));
    }
}
